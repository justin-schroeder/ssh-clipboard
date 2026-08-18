use std::collections::{HashSet, VecDeque};
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
use crate::daemon::{self, Status};
use crate::model::{Direction, MonitorEvent, human_bytes};

use super::{ACCENT, CYAN, GREEN, MUTED, PANEL, RED, SOFT, YELLOW, clean_truncate};

enum UiMessage {
    Event(MonitorEvent),
    Status(Status),
    Error(String),
}

struct MonitorApp {
    config: Config,
    receiver: Receiver<UiMessage>,
    status: Option<Status>,
    events: VecDeque<MonitorEvent>,
    error: Option<String>,
    paused: bool,
    sent: u64,
    received: u64,
    quit: bool,
}

impl MonitorApp {
    fn new(config: Config, receiver: Receiver<UiMessage>) -> Self {
        Self {
            config,
            receiver,
            status: None,
            events: VecDeque::new(),
            error: None,
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
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn on_message(&mut self, message: UiMessage) {
        match message {
            UiMessage::Event(event) => {
                self.error = None;
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
            UiMessage::Status(status) => self.status = Some(status),
            UiMessage::Error(error) => self.error = Some(error),
        }
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let [header, peers, activity, footer] = Layout::vertical([
            Constraint::Length(4),
            Constraint::Length(7),
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
        let mut peer_names = self
            .config
            .peers
            .iter()
            .map(|peer| peer.name.clone())
            .collect::<Vec<_>>();
        for peer in &connected {
            if !peer_names.contains(peer) {
                peer_names.push(peer.clone());
            }
        }
        peer_names.sort();
        let desired_version = self.status.as_ref().map(|status| status.desired_version.as_str());
        let mut spans = vec![
            Span::styled("● ", Style::new().fg(GREEN)),
            Span::styled(self.config.node_name.clone(), Style::new().fg(SOFT).bold()),
            Span::styled("  this machine", Style::new().fg(MUTED)),
        ];
        for peer in peer_names {
            spans.push(Span::styled("     │     ", Style::new().fg(PANEL)));
            let is_connected = connected.contains(&peer);
            let peer_version = self.status.as_ref().and_then(|status| {
                status
                    .peers
                    .iter()
                    .find(|status| status.name == peer)
                    .and_then(|status| status.version.as_deref())
            });
            let is_current =
                is_connected && peer_version.is_some_and(|version| Some(version) == desired_version);
            let color = if is_current {
                GREEN
            } else if is_connected {
                YELLOW
            } else {
                RED
            };
            spans.push(Span::styled(
                if is_connected { "● " } else { "○ " },
                Style::new().fg(color),
            ));
            spans.push(Span::styled(peer, Style::new().fg(SOFT).bold()));
            spans.push(Span::styled(
                if !is_connected {
                    "  reconnecting".to_owned()
                } else if is_current {
                    format!("  connected · v{}", peer_version.unwrap_or("legacy"))
                } else {
                    format!("  outdated · {}", peer_version.unwrap_or("legacy"))
                },
                Style::new().fg(if is_current { MUTED } else { color }),
            ));
        }
        let backend = self
            .status
            .as_ref()
            .map_or("detecting", |status| status.clipboard_backend.as_str());
        let version = self.status.as_ref().map_or("detecting", |status| {
            if status.version == status.desired_version {
                status.version.as_str()
            } else {
                status.desired_version.as_str()
            }
        });
        let block = panel("  Peers  ");
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(spans),
                Line::raw(""),
                Line::from(vec![
                    muted("backend "),
                    Span::styled(backend.to_owned(), Style::new().fg(SOFT)),
                    muted("     version "),
                    Span::styled(version.to_owned(), Style::new().fg(SOFT)),
                    muted("     sent "),
                    Span::styled(human_bytes(self.sent), Style::new().fg(SOFT)),
                    muted("     received "),
                    Span::styled(human_bytes(self.received), Style::new().fg(SOFT)),
                ]),
            ]),
            inner,
        );
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
        if let Ok(status) = daemon::query_status().await {
            let _ = sender.send(UiMessage::Status(status));
        }
        tokio::select! {
            () = tokio::time::sleep(Duration::from_secs(1)) => {}
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { return; }
            }
        }
    }
}

pub async fn run_monitor(config: Config) -> Result<()> {
    let handle = Handle::current();
    let (sender, receiver) = mpsc::channel();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    handle.spawn(monitor_feed(sender.clone(), shutdown_rx.clone()));
    handle.spawn(status_feed(sender, shutdown_rx));
    tokio::task::spawn_blocking(move || {
        ratatui::run(|terminal| MonitorApp::new(config, receiver).run(terminal))
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
        let mut app = MonitorApp::new(config.clone(), receiver);
        app.on_message(UiMessage::Status(Status {
            running: true,
            node_id: config.node_id,
            node_name: config.node_name,
            clipboard_backend: "NSPasteboard".into(),
            version: crate::update::CURRENT_VERSION.into(),
            desired_version: crate::update::CURRENT_VERSION.into(),
            connected_peers: vec!["server".into()],
            peers: vec![crate::daemon::PeerStatus {
                node_id: Uuid::new_v4(),
                name: "server".into(),
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
        assert!(rendered.contains("server  connected"));
        assert!(rendered.contains("design.pdf"));
        assert!(rendered.contains("4.0 KiB"));
    }
}
