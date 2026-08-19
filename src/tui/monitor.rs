use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use anyhow::{Context, Result};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table};
use ratatui::{DefaultTerminal, Frame};
use tokio::io::AsyncBufReadExt;
use tokio::runtime::Handle;
use tokio::sync::watch;

use crate::config::Config;
use crate::daemon::{self, PeerStatus, Status, UpdateNotification};
use crate::model::{Direction, MonitorEvent, human_bytes};
use crate::update;

use super::{ACCENT, CYAN, GREEN, MUTED, PANEL, RED, SOFT, YELLOW, clean_truncate};

enum UiMessage {
    Event(MonitorEvent),
    Status(Status),
    Offline(String),
    Error(String),
    UpdateNotified(UpdateNotification),
    UpdateFailed(String),
}

enum MonitorCommand {
    NotifyUpdates,
}

struct MonitorApp {
    config: Config,
    receiver: Receiver<UiMessage>,
    commands: tokio::sync::mpsc::UnboundedSender<MonitorCommand>,
    status: Option<Status>,
    peer_statuses: HashMap<String, PeerStatus>,
    events: VecDeque<MonitorEvent>,
    error: Option<String>,
    update_notice: Option<String>,
    update_pending: bool,
    paused: bool,
    sent: u64,
    received: u64,
    quit: bool,
}

impl MonitorApp {
    fn new(
        config: Config,
        receiver: Receiver<UiMessage>,
        commands: tokio::sync::mpsc::UnboundedSender<MonitorCommand>,
    ) -> Self {
        Self {
            config,
            receiver,
            commands,
            status: None,
            peer_statuses: HashMap::new(),
            events: VecDeque::new(),
            error: None,
            update_notice: None,
            update_pending: false,
            paused: false,
            sent: 0,
            received: 0,
            quit: false,
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.quit {
            while let Ok(message) = self.receiver.try_recv() {
                self.on_message(message);
            }
            terminal.draw(|frame| self.render(frame))?;
            if event::poll(Duration::from_millis(80))?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
                    KeyCode::Char('p' | ' ') => self.paused = !self.paused,
                    KeyCode::Char('c') => {
                        self.events.clear();
                        self.sent = 0;
                        self.received = 0;
                    }
                    KeyCode::Char('u') if !self.update_pending => {
                        self.update_pending = true;
                        self.update_notice =
                            Some("Checking npm latest and notifying connected peers…".into());
                        if self.commands.send(MonitorCommand::NotifyUpdates).is_err() {
                            self.update_pending = false;
                            self.update_notice = Some("Could not contact the update worker.".into());
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn on_message(&mut self, message: UiMessage) {
        match message {
            UiMessage::Event(event) => {
                if self.paused {
                    return;
                }
                match event.direction {
                    Direction::Send => self.sent = self.sent.saturating_add(event.total_bytes()),
                    Direction::Receive => {
                        self.received = self.received.saturating_add(event.total_bytes());
                    }
                    Direction::Local => {}
                }
                self.events.push_front(event);
                self.events.truncate(200);
            }
            UiMessage::Status(status) => {
                for peer in &status.peers {
                    self.peer_statuses.insert(peer.name.clone(), peer.clone());
                }
                self.status = Some(status);
                self.error = None;
            }
            UiMessage::Offline(error) => {
                self.status = None;
                self.error = Some(error);
            }
            UiMessage::Error(error) => self.error = Some(error),
            UiMessage::UpdateNotified(notification) => {
                self.update_pending = false;
                let unknown = if notification.version_unknown_peers == 0 {
                    String::new()
                } else {
                    format!(
                        "; {} connected peer{} cannot receive update events",
                        notification.version_unknown_peers,
                        if notification.version_unknown_peers == 1 {
                            ""
                        } else {
                            "s"
                        }
                    )
                };
                self.update_notice = Some(format!(
                    "Announced v{} to {} peer{}{}; npm check queued.",
                    notification.version,
                    notification.notified_peers,
                    if notification.notified_peers == 1 { "" } else { "s" },
                    unknown
                ));
            }
            UiMessage::UpdateFailed(error) => {
                self.update_pending = false;
                self.update_notice = Some(format!("Update notification failed: {error}"));
            }
        }
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let peer_height = u16::try_from(self.peer_names().len())
            .unwrap_or(u16::MAX)
            .saturating_add(7)
            .clamp(8, area.height.saturating_sub(14).max(8));
        let [header, peers, activity, footer] = Layout::vertical([
            Constraint::Length(4),
            Constraint::Length(peer_height),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .areas(area);
        self.render_header(frame, header);
        let width = area.width.saturating_sub(6).min(120);
        let [peers] = Layout::horizontal([Constraint::Length(width)])
            .flex(Flex::Center)
            .areas(peers);
        let [activity] = Layout::horizontal([Constraint::Length(width)])
            .flex(Flex::Center)
            .areas(activity);
        self.render_peers(frame, peers);
        self.render_activity(frame, activity);
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                key("p"),
                muted(" pause   •   "),
                key("c"),
                muted(" clear   •   "),
                key("u"),
                muted(" notify updates   •   "),
                key("q"),
                muted(" close"),
            ]))
            .alignment(Alignment::Center),
            footer,
        );
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let live = if self.paused {
            Span::styled("● PAUSED", Style::new().fg(YELLOW).bold())
        } else if self.status.as_ref().is_some_and(|status| status.running) {
            Span::styled("● LIVE", Style::new().fg(GREEN).bold())
        } else {
            Span::styled("● OFFLINE", Style::new().fg(RED).bold())
        };
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("ssh", Style::new().fg(ACCENT).bold()),
                    Span::styled(" ◇ ", Style::new().fg(SOFT)),
                    Span::styled("clipboard", Style::new().fg(ACCENT).bold()),
                    Span::raw("   "),
                    live,
                ]),
                Line::styled(
                    "native clipboard  •  persistent SSH  •  zero cloud hops",
                    Style::new().fg(MUTED),
                ),
            ])
            .alignment(Alignment::Center),
            area,
        );
    }

    fn render_peers(&self, frame: &mut Frame, area: Rect) {
        let connected = self
            .status
            .as_ref()
            .map(|status| status.connected_peers.iter().cloned().collect::<HashSet<_>>())
            .unwrap_or_default();
        let peer_names = self.peer_names();
        let desired_version = self
            .status
            .as_ref()
            .and_then(|status| known_version(&status.desired_version));
        let local_name = self.status.as_ref().map_or_else(
            || self.config.node_name.as_str(),
            |status| {
                if status.machine_name.is_empty() {
                    status.node_name.as_str()
                } else {
                    status.machine_name.as_str()
                }
            },
        );
        let local_version = self
            .status
            .as_ref()
            .and_then(|status| known_version(&status.version));
        let local_state = match (local_version, desired_version) {
            (Some(installed), Some(desired)) if installed == desired => "current",
            (Some(_), Some(_)) => "updating",
            _ => "detecting",
        };
        let mut rows = vec![Row::new(vec![
            Cell::from(Line::from(vec![
                Span::styled("● ", Style::new().fg(GREEN)),
                Span::styled(local_name.to_owned(), Style::new().fg(SOFT).bold()),
            ])),
            Cell::from("this machine").style(Style::new().fg(MUTED)),
            Cell::from(version_label(local_version)).style(Style::new().fg(SOFT)),
            Cell::from(version_label(desired_version)).style(Style::new().fg(SOFT)),
            Cell::from(local_state).style(Style::new().fg(if local_state == "current" {
                GREEN
            } else {
                YELLOW
            })),
        ])];
        for peer in peer_names {
            let is_connected = connected.contains(&peer);
            let live_status = self
                .status
                .as_ref()
                .and_then(|status| status.peers.iter().find(|status| status.name == peer));
            let peer_status = live_status.or_else(|| self.peer_statuses.get(&peer));
            let peer_version = peer_status.and_then(|status| status.version.as_deref());
            let peer_desired = peer_status.and_then(|status| status.desired_version.as_deref());
            let target_version =
                peer_desired.or_else(|| peer_version.and_then(known_version).and(desired_version));
            let peer_label = peer_status
                .and_then(|status| status.machine_name.as_deref())
                .filter(|name| !name.is_empty())
                .unwrap_or(&peer);
            let (state, color) = peer_update_state(is_connected, peer_version, peer_desired, desired_version);
            rows.push(Row::new(vec![
                Cell::from(Line::from(vec![
                    Span::styled(if is_connected { "● " } else { "○ " }, Style::new().fg(color)),
                    Span::styled(peer_label.to_owned(), Style::new().fg(SOFT).bold()),
                ])),
                Cell::from(if is_connected { "connected" } else { "reconnecting" })
                    .style(Style::new().fg(if is_connected { MUTED } else { RED })),
                Cell::from(version_label(peer_version)).style(Style::new().fg(SOFT)),
                Cell::from(version_label(target_version)).style(Style::new().fg(SOFT)),
                Cell::from(state).style(Style::new().fg(color)),
            ]));
        }
        let backend = self
            .status
            .as_ref()
            .map_or("detecting", |status| status.clipboard_backend.as_str());
        let version = self.status.as_ref().map_or_else(
            || "detecting".to_owned(),
            |status| {
                if known_version(&status.version).is_none() {
                    "unknown".into()
                } else if status.version == status.desired_version {
                    format!("v{}", status.version)
                } else {
                    format!("v{} → v{}", status.version, status.desired_version)
                }
            },
        );
        let block = panel("  Peers  ");
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let [table_area, details_area] =
            Layout::vertical([Constraint::Min(3), Constraint::Length(2)]).areas(inner);
        let table = Table::new(
            rows,
            [
                Constraint::Min(22),
                Constraint::Length(13),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Min(18),
            ],
        );
        let header = Row::new(["MACHINE", "CONNECTION", "INSTALLED", "TARGET", "UPDATE"])
            .style(Style::new().fg(MUTED).add_modifier(Modifier::BOLD))
            .bottom_margin(1);
        frame.render_widget(table.header(header).column_spacing(1), table_area);
        let mut details = vec![Line::from(vec![
            muted("backend "),
            Span::styled(backend.to_owned(), Style::new().fg(SOFT)),
            muted("   local "),
            Span::styled(version, Style::new().fg(SOFT)),
            muted("   sent "),
            Span::styled(human_bytes(self.sent), Style::new().fg(SOFT)),
            muted("   received "),
            Span::styled(human_bytes(self.received), Style::new().fg(SOFT)),
        ])];
        if let Some(notice) = &self.update_notice {
            details.push(Line::styled(
                clean_truncate(notice, usize::from(details_area.width)),
                Style::new().fg(if self.update_pending { YELLOW } else { CYAN }),
            ));
        }
        frame.render_widget(Paragraph::new(details), details_area);
    }

    fn peer_names(&self) -> Vec<String> {
        let mut peers = self.status.as_ref().map_or_else(
            || {
                self.config
                    .peers
                    .iter()
                    .map(|peer| peer.name.clone())
                    .collect::<Vec<_>>()
            },
            |status| status.configured_peers.clone(),
        );
        if let Some(status) = &self.status {
            for peer in &status.connected_peers {
                if !peers.contains(peer) {
                    peers.push(peer.clone());
                }
            }
        }
        peers.sort();
        peers.dedup();
        peers
    }

    fn render_activity(&self, frame: &mut Frame, area: Rect) {
        let block = panel("  Clipboard activity  ");
        let inner = block.inner(area);
        frame.render_widget(block, area);
        if let Some(error) = &self.error {
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled("Connection unavailable  ", Style::new().fg(RED).bold()),
                        Span::styled(
                            clean_truncate(error, usize::from(inner.width.saturating_sub(26))),
                            Style::new().fg(SOFT),
                        ),
                    ]),
                    Line::raw(""),
                    Line::styled(
                        "The service keeps retrying. Run `ssh-clipboard service restart` if needed.",
                        Style::new().fg(MUTED),
                    ),
                ]),
                inner,
            );
            return;
        }
        if self.events.is_empty() {
            frame.render_widget(
                Paragraph::new(vec![
                    Line::styled("Waiting for the next copy…", Style::new().fg(MUTED)),
                    Line::raw(""),
                    Line::styled(
                        "Copy text, an image, files, or rich content on any connected machine.",
                        Style::new().fg(SOFT),
                    ),
                ]),
                inner,
            );
            return;
        }
        let rows = self.events.iter().map(|event| {
            let (flow, color) = flow(event);
            Row::new(vec![
                Cell::from(time_of_day(event.timestamp_millis)).style(Style::new().fg(MUTED)),
                Cell::from(flow).style(Style::new().fg(color).bold()),
                Cell::from(clean_truncate(
                    &event.preview,
                    usize::from(inner.width.saturating_sub(54)),
                ))
                .style(Style::new().fg(SOFT)),
                Cell::from(human_bytes(event.total_bytes())).style(Style::new().fg(MUTED)),
                Cell::from(format!("{}", event.representations.len())).style(Style::new().fg(MUTED)),
            ])
        });
        let header = Row::new(["TIME", "FLOW", "CONTENT", "SIZE", "FORMATS"])
            .style(Style::new().fg(MUTED).add_modifier(Modifier::BOLD))
            .bottom_margin(1);
        let table = Table::new(
            rows,
            [
                Constraint::Length(13),
                Constraint::Length(20),
                Constraint::Min(12),
                Constraint::Length(10),
                Constraint::Length(7),
            ],
        )
        .header(header)
        .column_spacing(1);
        frame.render_widget(table, inner);
    }
}

fn version_label(version: Option<&str>) -> String {
    version
        .and_then(known_version)
        .map_or_else(|| "unknown".into(), |version| format!("v{version}"))
}

fn known_version(version: &str) -> Option<&str> {
    (!version.is_empty() && version != "legacy").then_some(version)
}

fn peer_update_state(
    connected: bool,
    installed: Option<&str>,
    peer_desired: Option<&str>,
    local_desired: Option<&str>,
) -> (&'static str, ratatui::style::Color) {
    if !connected {
        return ("offline", RED);
    }
    let Some(installed) = installed.and_then(known_version) else {
        return ("version unknown · setup required", YELLOW);
    };
    let peer_desired = peer_desired.and_then(known_version);
    let local_desired = local_desired.and_then(known_version);
    if peer_desired.is_some_and(|desired| update::newer_version(installed, desired)) {
        return ("updating", YELLOW);
    }
    if local_desired.is_some_and(|desired| update::newer_version(installed, desired)) {
        return ("outdated · press u", YELLOW);
    }
    if local_desired.is_some_and(|desired| update::newer_version(desired, installed)) {
        return ("ahead", CYAN);
    }
    ("current", GREEN)
}

fn panel(title: &'static str) -> Block<'static> {
    Block::new()
        .title(Line::styled(title, Style::new().fg(ACCENT).bold()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(PANEL))
        .padding(ratatui::widgets::Padding::horizontal(2))
}

fn flow(event: &MonitorEvent) -> (String, ratatui::style::Color) {
    match event.direction {
        Direction::Local => ("◆ copied here".into(), ACCENT),
        Direction::Send => (
            format!(
                "→ {}",
                clean_truncate(event.peer.as_deref().unwrap_or("peer"), 16)
            ),
            CYAN,
        ),
        Direction::Receive => (
            format!(
                "← {}",
                clean_truncate(event.peer.as_deref().unwrap_or("peer"), 16)
            ),
            GREEN,
        ),
    }
}

fn time_of_day(timestamp_millis: u64) -> String {
    let day = timestamp_millis % 86_400_000;
    let hours = day / 3_600_000;
    let minutes = (day / 60_000) % 60;
    let seconds = (day / 1_000) % 60;
    let millis = day % 1_000;
    format!("{hours:02}:{minutes:02}:{seconds:02}.{millis:03}")
}

fn key(value: &'static str) -> Span<'static> {
    Span::styled(value, Style::new().fg(CYAN).bold())
}

fn muted(value: &'static str) -> Span<'static> {
    Span::styled(value, Style::new().fg(MUTED))
}

async fn monitor_feed(sender: Sender<UiMessage>, mut shutdown: watch::Receiver<bool>) {
    loop {
        match daemon::connect_monitor().await {
            Ok(mut reader) => {
                let mut line = String::new();
                loop {
                    line.clear();
                    tokio::select! {
                        result = reader.read_line(&mut line) => match result {
                            Ok(0) => break,
                            Ok(_) => match serde_json::from_str::<MonitorEvent>(&line) {
                                Ok(event) => { let _ = sender.send(UiMessage::Event(event)); }
                                Err(error) => { let _ = sender.send(UiMessage::Error(error.to_string())); }
                            },
                            Err(error) => {
                                let _ = sender.send(UiMessage::Error(error.to_string()));
                                break;
                            }
                        },
                        changed = shutdown.changed() => {
                            if changed.is_err() || *shutdown.borrow() { return; }
                        }
                    }
                }
            }
            Err(error) => {
                let _ = sender.send(UiMessage::Error(error.to_string()));
            }
        }
        tokio::select! {
            () = tokio::time::sleep(Duration::from_secs(1)) => {}
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { return; }
            }
        }
    }
}

async fn status_feed(sender: Sender<UiMessage>, mut shutdown: watch::Receiver<bool>) {
    loop {
        match daemon::query_status().await {
            Ok(status) => {
                let _ = sender.send(UiMessage::Status(status));
            }
            Err(error) => {
                let _ = sender.send(UiMessage::Offline(error.to_string()));
            }
        }
        tokio::select! {
            () = tokio::time::sleep(Duration::from_secs(1)) => {}
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { return; }
            }
        }
    }
}

async fn command_feed(
    sender: Sender<UiMessage>,
    mut commands: tokio::sync::mpsc::UnboundedReceiver<MonitorCommand>,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            command = commands.recv() => match command {
                Some(MonitorCommand::NotifyUpdates) => {
                    let message = match daemon::notify_updates().await {
                        Ok(notification) => UiMessage::UpdateNotified(notification),
                        Err(error) => UiMessage::UpdateFailed(error.to_string()),
                    };
                    let _ = sender.send(message);
                }
                None => return,
            },
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    return;
                }
            }
        }
    }
}

pub async fn run_monitor(config: Config) -> Result<()> {
    let handle = Handle::current();
    let (sender, receiver) = mpsc::channel();
    let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    handle.spawn(monitor_feed(sender.clone(), shutdown_rx.clone()));
    handle.spawn(status_feed(sender.clone(), shutdown_rx.clone()));
    handle.spawn(command_feed(sender, command_rx, shutdown_rx));
    tokio::task::spawn_blocking(move || {
        ratatui::run(|terminal| MonitorApp::new(config, receiver, command_tx).run(terminal))
    })
    .await
    .context("monitor TUI task failed")??;
    let _ = shutdown_tx.send(true);
    Ok(())
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use uuid::Uuid;

    use super::*;
    use crate::model::{RepresentationInfo, now_millis};

    fn monitor_app(config: Config, receiver: Receiver<UiMessage>) -> MonitorApp {
        let (commands, _command_receiver) = tokio::sync::mpsc::unbounded_channel();
        MonitorApp::new(config, receiver, commands)
    }

    #[test]
    fn monitor_renders_peer_health_and_native_activity() {
        let (sender, receiver) = mpsc::channel();
        let mut config = Config {
            node_name: "local".into(),
            ..Config::default()
        };
        config.peers.push(crate::config::PeerConfig {
            name: "server".into(),
            ssh_command: "ssh server".into(),
        });
        let mut app = monitor_app(config.clone(), receiver);
        app.on_message(UiMessage::Status(Status {
            running: true,
            node_id: config.node_id,
            node_name: config.node_name,
            machine_name: "local-machine.local".into(),
            clipboard_backend: "NSPasteboard".into(),
            version: crate::update::CURRENT_VERSION.into(),
            desired_version: crate::update::CURRENT_VERSION.into(),
            configured_peers: vec!["server".into()],
            connected_peers: vec!["server".into()],
            peers: vec![crate::daemon::PeerStatus {
                node_id: Uuid::new_v4(),
                name: "server".into(),
                machine_name: Some("server-machine.local".into()),
                version: Some(crate::update::CURRENT_VERSION.into()),
                desired_version: Some(crate::update::CURRENT_VERSION.into()),
            }],
        }));
        app.on_message(UiMessage::Event(MonitorEvent {
            timestamp_millis: now_millis(),
            direction: Direction::Receive,
            peer: Some("server".into()),
            clip_id: Uuid::new_v4(),
            origin: Uuid::new_v4(),
            preview: "design.pdf".into(),
            representations: vec![RepresentationInfo {
                item: 0,
                format: "application/pdf".into(),
                bytes: 4096,
            }],
        }));
        drop(sender);
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.render(frame)).unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert!(rendered.contains("● LIVE"));
        assert!(rendered.contains("local-machine.local"));
        assert!(rendered.contains("server-machine.local"));
        assert!(rendered.contains(&format!("v{}", crate::update::CURRENT_VERSION)));
        assert!(rendered.contains("current"));
        assert!(rendered.contains("design.pdf"));
        assert!(rendered.contains("4.0 KiB"));
    }

    #[test]
    fn monitor_drops_stale_status_when_the_daemon_disappears() {
        let (_sender, receiver) = mpsc::channel();
        let config = Config {
            node_name: "local".into(),
            ..Config::default()
        };
        let mut app = monitor_app(config.clone(), receiver);
        app.on_message(UiMessage::Status(Status {
            running: true,
            node_id: config.node_id,
            node_name: config.node_name,
            machine_name: "local-machine.local".into(),
            clipboard_backend: "NSPasteboard".into(),
            version: crate::update::CURRENT_VERSION.into(),
            desired_version: crate::update::CURRENT_VERSION.into(),
            configured_peers: Vec::new(),
            connected_peers: Vec::new(),
            peers: Vec::new(),
        }));
        app.on_message(UiMessage::Offline("daemon socket unavailable".into()));

        assert!(app.status.is_none());
        assert_eq!(app.error.as_deref(), Some("daemon socket unavailable"));

        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.render(frame)).unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert!(rendered.contains("● OFFLINE"));
        assert!(rendered.contains("Connection unavailable"));
    }

    #[test]
    fn monitor_shows_running_and_desired_versions_during_an_update() {
        let (_sender, receiver) = mpsc::channel();
        let mut config = Config {
            node_name: "local".into(),
            ..Config::default()
        };
        config.peers.push(crate::config::PeerConfig {
            name: "server".into(),
            ssh_command: "ssh server".into(),
        });
        let mut app = monitor_app(config.clone(), receiver);
        app.on_message(UiMessage::Status(Status {
            running: true,
            node_id: config.node_id,
            node_name: config.node_name,
            machine_name: "local-machine.local".into(),
            clipboard_backend: "NSPasteboard".into(),
            version: crate::update::CURRENT_VERSION.into(),
            desired_version: "9.0.0".into(),
            configured_peers: vec!["server".into()],
            connected_peers: vec!["server".into()],
            peers: vec![crate::daemon::PeerStatus {
                node_id: Uuid::new_v4(),
                name: "server".into(),
                machine_name: Some("server-machine.local".into()),
                version: Some(crate::update::CURRENT_VERSION.into()),
                desired_version: Some("9.0.0".into()),
            }],
        }));

        let backend = TestBackend::new(160, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.render(frame)).unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert!(rendered.contains("updating"));
        assert!(rendered.contains(&format!("v{} → v9.0.0", crate::update::CURRENT_VERSION)));
    }

    #[test]
    fn monitor_lists_unknown_and_disconnected_peer_versions_without_legacy_label() {
        let (_sender, receiver) = mpsc::channel();
        let mut config = Config {
            node_name: "local".into(),
            ..Config::default()
        };
        for name in ["modern", "older", "sleeping"] {
            config.peers.push(crate::config::PeerConfig {
                name: name.into(),
                ssh_command: format!("ssh {name}"),
            });
        }
        let mut app = monitor_app(config.clone(), receiver);
        app.on_message(UiMessage::Status(Status {
            running: true,
            node_id: config.node_id,
            node_name: config.node_name,
            machine_name: "local-machine.local".into(),
            clipboard_backend: "NSPasteboard".into(),
            version: crate::update::CURRENT_VERSION.into(),
            desired_version: crate::update::CURRENT_VERSION.into(),
            configured_peers: vec!["modern".into(), "older".into(), "sleeping".into()],
            connected_peers: vec!["modern".into(), "older".into()],
            peers: vec![
                PeerStatus {
                    node_id: Uuid::new_v4(),
                    name: "modern".into(),
                    machine_name: Some("modern-machine.local".into()),
                    version: Some(crate::update::CURRENT_VERSION.into()),
                    desired_version: Some(crate::update::CURRENT_VERSION.into()),
                },
                PeerStatus {
                    node_id: Uuid::new_v4(),
                    name: "older".into(),
                    machine_name: Some("older-machine.local".into()),
                    version: None,
                    desired_version: None,
                },
            ],
        }));

        let backend = TestBackend::new(150, 34);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.render(frame)).unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert!(rendered.contains("MACHINE"));
        assert!(rendered.contains("INSTALLED"));
        assert!(rendered.contains("TARGET"));
        assert!(rendered.contains("modern-machine.local"));
        assert!(rendered.contains("older-machine.local"));
        assert!(rendered.contains("version unknown · setup required"));
        assert!(rendered.contains("sleeping"));
        assert!(rendered.contains("reconnecting"));
        assert!(!rendered.contains("legacy"));
        assert!(rendered.contains("u notify updates"));
    }

    #[test]
    fn monitor_reports_update_notifications() {
        let (_sender, receiver) = mpsc::channel();
        let mut app = monitor_app(Config::default(), receiver);
        app.update_pending = true;
        app.on_message(UiMessage::UpdateNotified(UpdateNotification {
            version: "1.2.3".into(),
            notified_peers: 2,
            version_unknown_peers: 1,
        }));

        assert!(!app.update_pending);
        let notice = app.update_notice.as_deref().unwrap();
        assert!(notice.contains("Announced v1.2.3 to 2 peers"));
        assert!(notice.contains("1 connected peer cannot receive update events"));
    }
}
