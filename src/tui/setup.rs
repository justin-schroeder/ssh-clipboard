use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Gauge, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};
use tokio::runtime::Handle;

use crate::config::{Config, PeerConfig};
use crate::deploy;
use crate::ssh::{self, ProbeResult};

use super::{ACCENT, CYAN, GREEN, MUTED, PANEL, RED, SOFT, clean_truncate};

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Welcome,
    Entry,
    Verifying,
    Confirmed,
    Installing,
    Ready,
    Failed,
}

#[derive(Clone, Debug)]
struct VerifiedPeer {
    command: String,
    probe: ProbeResult,
}

enum UiMessage {
    Verified {
        command: String,
        result: Result<ProbeResult, String>,
    },
    Progress {
        peer: String,
        detail: String,
    },
    Installed(Result<(), String>),
}

struct SetupApp {
    handle: Handle,
    config: Config,
    stage: Stage,
    input: String,
    peers: Vec<VerifiedPeer>,
    active_peer: String,
    detail: String,
    completed: Vec<String>,
    error: Option<String>,
    receiver: Receiver<UiMessage>,
    sender: Sender<UiMessage>,
    spinner: usize,
    last_tick: Instant,
    quit: bool,
}

impl SetupApp {
    fn new(handle: Handle, config: Config) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            handle,
            config,
            stage: Stage::Welcome,
            input: String::new(),
            peers: Vec::new(),
            active_peer: String::new(),
            detail: String::new(),
            completed: Vec::new(),
            error: None,
            receiver,
            sender,
            spinner: 0,
            last_tick: Instant::now(),
            quit: false,
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.quit {
            while let Ok(message) = self.receiver.try_recv() {
                self.on_message(message);
            }
            if self.last_tick.elapsed() >= Duration::from_millis(80) {
                self.spinner = (self.spinner + 1) % SPINNER.len();
                self.last_tick = Instant::now();
            }
            terminal.draw(|frame| self.render(frame))?;
            if event::poll(Duration::from_millis(40))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key(key),
                    Event::Paste(value) if self.stage == Stage::Entry => self.input.push_str(&value),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn on_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }
        match self.stage {
            Stage::Welcome => {
                if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
                    self.stage = Stage::Entry;
                }
            }
            Stage::Entry => self.on_entry_key(key),
            Stage::Confirmed => match key.code {
                KeyCode::Char('a') => {
                    self.input.clear();
                    self.stage = Stage::Entry;
                }
                KeyCode::Enter | KeyCode::Char('i') => self.begin_install(),
                _ => {}
            },
            Stage::Ready => {
                if matches!(key.code, KeyCode::Enter | KeyCode::Char('q')) {
                    self.quit = true;
                }
            }
            Stage::Failed if key.code == KeyCode::Char('r') => self.begin_install(),
            Stage::Verifying | Stage::Installing | Stage::Failed => {}
        }
    }

    fn on_entry_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                let command = self.input.trim().to_owned();
                if command.is_empty() {
                    if !self.peers.is_empty() {
                        self.begin_install();
                    }
                    return;
                }
                self.error = None;
                self.stage = Stage::Verifying;
                let sender = self.sender.clone();
                self.handle.spawn(async move {
                    let result = ssh::probe(&command).await.map_err(|error| error.to_string());
                    let _ = sender.send(UiMessage::Verified { command, result });
                });
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.input.push(character);
            }
            _ => {}
        }
    }

    fn begin_install(&mut self) {
        if self.peers.is_empty() {
            return;
        }
        self.stage = Stage::Installing;
        self.error = None;
        self.completed.clear();
        self.active_peer.clear();
        self.detail = "Preparing private peer configuration".into();
        let sender = self.sender.clone();
        let config = self.config.clone();
        let peers = self.peers.clone();
        self.handle.spawn(async move {
            let result = install_all(config, peers, sender.clone())
                .await
                .map_err(|error| error.to_string());
            let _ = sender.send(UiMessage::Installed(result));
        });
    }

    fn on_message(&mut self, message: UiMessage) {
        match message {
            UiMessage::Verified { command, result } => match result {
                Ok(probe) => {
                    self.peers.push(VerifiedPeer { command, probe });
                    self.stage = Stage::Confirmed;
                    self.error = None;
                }
                Err(error) => {
                    self.error = Some(error);
                    self.stage = Stage::Entry;
                }
            },
            UiMessage::Progress { peer, detail } => {
                if detail == "Installed and running" && !self.completed.contains(&peer) {
                    self.completed.push(peer.clone());
                }
                self.active_peer = peer;
                self.detail = detail;
            }
            UiMessage::Installed(result) => match result {
                Ok(()) => self.stage = Stage::Ready,
                Err(error) => {
                    self.error = Some(error);
                    self.stage = Stage::Failed;
                }
            },
        }
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let [header, steps, body, footer] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(13),
            Constraint::Length(2),
        ])
        .areas(area);
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("ssh", Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)),
                Span::styled(" ◇ ", Style::new().fg(SOFT)),
                Span::styled("clipboard", Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)),
            ]))
            .alignment(Alignment::Center),
            header,
        );
        frame.render_widget(self.steps(), steps);
        let body_width = area.width.saturating_sub(6).min(92);
        let [body_area] = Layout::horizontal([Constraint::Length(body_width)])
            .flex(Flex::Center)
            .areas(body);
        self.render_body(frame, body_area);
        frame.render_widget(Paragraph::new(self.help()).alignment(Alignment::Center), footer);
    }

    fn steps(&self) -> Paragraph<'static> {
        let current = match self.stage {
            Stage::Welcome => 0,
            Stage::Entry | Stage::Verifying | Stage::Confirmed => 1,
            Stage::Installing | Stage::Failed => 2,
            Stage::Ready => 3,
        };
        let labels = ["WELCOME", "PEERS", "INSTALL", "READY"];
        let mut spans = Vec::new();
        for (index, label) in labels.into_iter().enumerate() {
            let (mark, color) = match index.cmp(&current) {
                std::cmp::Ordering::Less => ("✓", GREEN),
                std::cmp::Ordering::Equal => ("●", CYAN),
                std::cmp::Ordering::Greater => ("○", MUTED),
            };
            spans.push(Span::styled(
                format!("{mark} {label}"),
                Style::new().fg(color).add_modifier(Modifier::BOLD),
            ));
            if index < 3 {
                spans.push(Span::styled(" ──── ", Style::new().fg(MUTED)));
            }
        }
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center)
    }

    fn render_body(&self, frame: &mut Frame, area: Rect) {
        let title = match self.stage {
            Stage::Welcome => "  Your clipboard, without borders  ",
            Stage::Entry | Stage::Verifying => "  Add a passwordless SSH peer  ",
            Stage::Confirmed => "  Peer verified  ",
            Stage::Installing => "  Installing  ",
            Stage::Ready => "  Connected  ",
            Stage::Failed => "  Installation paused  ",
        };
        let block = Block::new()
            .title(Line::styled(title, Style::new().fg(ACCENT).bold()).centered())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(PANEL))
            .padding(ratatui::widgets::Padding::new(2, 2, 1, 1));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        match self.stage {
            Stage::Installing => self.render_installing(frame, inner),
            _ => frame.render_widget(Paragraph::new(self.body_text()).wrap(Wrap { trim: false }), inner),
        }
    }

    fn body_text(&self) -> Text<'static> {
        match self.stage {
            Stage::Welcome => Text::from(vec![
                Line::styled(
                    "Copy on one machine. Paste on another.",
                    Style::new().fg(SOFT).bold(),
                ),
                Line::raw(""),
                Line::styled(
                    "Text, images, files, rich text, and native clipboard formats move as original bytes over persistent encrypted SSH.",
                    Style::new().fg(SOFT),
                ),
                Line::raw(""),
                Line::from(vec![
                    Span::styled("● No cloud account", Style::new().fg(GREEN)),
                    Span::raw("     "),
                    Span::styled("● No conversion", Style::new().fg(GREEN)),
                    Span::raw("     "),
                    Span::styled("● Starts at login", Style::new().fg(GREEN)),
                ]),
            ]),
            Stage::Entry => {
                let visible = if self.input.is_empty() {
                    Span::styled("macbookserver   or   ssh user@host", Style::new().fg(MUTED))
                } else {
                    Span::styled(clean_truncate(&self.input, 74), Style::new().fg(SOFT))
                };
                let mut lines = vec![
                    Line::styled(
                        "Use the exact SSH command that already connects without prompting.",
                        Style::new().fg(MUTED),
                    ),
                    Line::raw(""),
                    Line::from(vec![
                        Span::styled(" ssh  ", Style::new().fg(CYAN).bold()),
                        visible,
                        Span::styled("▏", Style::new().fg(CYAN)),
                    ]),
                ];
                if let Some(error) = &self.error {
                    lines.push(Line::raw(""));
                    lines.push(Line::from(vec![
                        Span::styled("Couldn’t verify  ", Style::new().fg(RED).bold()),
                        Span::styled(clean_truncate(error, 72), Style::new().fg(SOFT)),
                    ]));
                }
                if !self.peers.is_empty() {
                    lines.push(Line::raw(""));
                    lines.push(Line::styled(
                        format!(
                            "✓ {} peer(s) ready — leave blank and press enter to install",
                            self.peers.len()
                        ),
                        Style::new().fg(GREEN),
                    ));
                }
                Text::from(lines)
            }
            Stage::Verifying => Text::from(vec![
                Line::from(vec![
                    Span::styled(SPINNER[self.spinner], Style::new().fg(CYAN)),
                    Span::styled("  Opening an encrypted connection…", Style::new().fg(SOFT).bold()),
                ]),
                Line::raw(""),
                Line::styled(clean_truncate(&self.input, 76), Style::new().fg(MUTED)),
                Line::raw(""),
                Line::styled(
                    "Password and keyboard-interactive prompts are disabled; verification fails safely.",
                    Style::new().fg(MUTED),
                ),
            ]),
            Stage::Confirmed => {
                let peer = self.peers.last().expect("confirmed stage has a peer");
                Text::from(vec![
                    Line::styled("✓  Connection verified", Style::new().fg(GREEN).bold()),
                    Line::raw(""),
                    Line::from(vec![
                        Span::styled(peer.probe.hostname.clone(), Style::new().fg(SOFT).bold()),
                        Span::styled(
                            format!("   {}/{}", peer.probe.os, peer.probe.arch),
                            Style::new().fg(MUTED),
                        ),
                    ]),
                    Line::styled(peer.command.clone(), Style::new().fg(MUTED)),
                    Line::raw(""),
                    Line::styled(
                        "The host is reachable without a password and ready for installation.",
                        Style::new().fg(SOFT),
                    ),
                ])
            }
            Stage::Ready => Text::from(vec![
                Line::styled("✓  Your clipboards are connected", Style::new().fg(GREEN).bold()),
                Line::raw(""),
                Line::styled(
                    format!("{} peer(s) configured", self.peers.len()),
                    Style::new().fg(SOFT).bold(),
                ),
                Line::styled(
                    "Copy normally on either machine. The destination’s native clipboard changes, so Raycast and other clipboard managers see it naturally.",
                    Style::new().fg(SOFT),
                ),
                Line::raw(""),
                Line::styled(
                    "Run ssh-clipboard monitor any time to watch activity and connection health.",
                    Style::new().fg(MUTED),
                ),
            ]),
            Stage::Failed => Text::from(vec![
                Line::styled("Installation paused", Style::new().fg(RED).bold()),
                Line::raw(""),
                Line::styled(
                    clean_truncate(self.error.as_deref().unwrap_or("Unknown error"), 140),
                    Style::new().fg(SOFT),
                ),
                Line::raw(""),
                Line::styled(
                    "Completed peers are safe to reinstall; setup is idempotent.",
                    Style::new().fg(MUTED),
                ),
            ]),
            Stage::Installing => Text::default(),
        }
    }

    fn render_installing(&self, frame: &mut Frame, area: Rect) {
        let [heading, list, gauge, note] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(3),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .areas(area);
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(SPINNER[self.spinner], Style::new().fg(CYAN)),
                Span::styled(
                    "  Building your private clipboard mesh",
                    Style::new().fg(SOFT).bold(),
                ),
            ])),
            heading,
        );
        let mut lines = self
            .completed
            .iter()
            .map(|peer| Line::styled(format!("✓  {peer}"), Style::new().fg(GREEN)))
            .collect::<Vec<_>>();
        if !self.active_peer.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("●  ", Style::new().fg(CYAN)),
                Span::styled(self.active_peer.clone(), Style::new().fg(SOFT).bold()),
                Span::styled(format!("   {}", self.detail), Style::new().fg(MUTED)),
            ]));
        }
        frame.render_widget(Paragraph::new(lines), list);
        let total = self.peers.len() + 1;
        let ratio = (self.completed.len() as f64 / total as f64).clamp(0.02, 1.0);
        frame.render_widget(
            Gauge::default()
                .ratio(ratio)
                .gauge_style(Style::new().fg(ACCENT).bg(PANEL))
                .label(format!("{} / {total}", self.completed.len())),
            gauge.inner(Margin::new(0, 0)),
        );
        frame.render_widget(
            Paragraph::new("Persistent channels keep updates instant—even for large images.")
                .style(Style::new().fg(MUTED)),
            note,
        );
    }

    fn help(&self) -> Line<'static> {
        let items: &[(&str, &str)] = match self.stage {
            Stage::Welcome => &[("enter", "begin"), ("ctrl+c", "quit")],
            Stage::Entry => &[("enter", "verify / install"), ("ctrl+c", "quit")],
            Stage::Confirmed => &[("enter", "install"), ("a", "add another"), ("ctrl+c", "quit")],
            Stage::Installing | Stage::Verifying => &[("ctrl+c", "cancel")],
            Stage::Ready => &[("enter", "close"), ("ssh-clipboard monitor", "watch activity")],
            Stage::Failed => &[("r", "retry"), ("ctrl+c", "quit")],
        };
        let mut spans = Vec::new();
        for (index, (key, description)) in items.iter().enumerate() {
            if index > 0 {
                spans.push(Span::styled("   •   ", Style::new().fg(MUTED)));
            }
            spans.push(Span::styled((*key).to_owned(), Style::new().fg(CYAN).bold()));
            spans.push(Span::styled(format!(" {description}"), Style::new().fg(MUTED)));
        }
        Line::from(spans)
    }
}

async fn install_all(config: Config, peers: Vec<VerifiedPeer>, sender: Sender<UiMessage>) -> Result<()> {
    let mut local = config;
    local.peers = peers
        .iter()
        .map(|peer| PeerConfig {
            name: peer.probe.hostname.clone(),
            ssh_command: peer.command.clone(),
        })
        .collect();
    local.save()?;
    for peer in &peers {
        let name = peer.probe.hostname.clone();
        let progress_sender = sender.clone();
        deploy::install_remote(&peer.command, &peer.probe, |_, detail| {
            let _ = progress_sender.send(UiMessage::Progress {
                peer: name.clone(),
                detail: detail.to_owned(),
            });
        })
        .await
        .with_context(|| format!("install {name}"))?;
        let _ = sender.send(UiMessage::Progress {
            peer: name,
            detail: "Installed and running".into(),
        });
    }
    let _ = sender.send(UiMessage::Progress {
        peer: local.node_name.clone(),
        detail: "Installing this machine’s service".into(),
    });
    deploy::install_local_service().await?;
    let _ = sender.send(UiMessage::Progress {
        peer: local.node_name,
        detail: "Installed and running".into(),
    });
    Ok(())
}

pub async fn run_setup(config: Config) -> Result<()> {
    let handle = Handle::current();
    tokio::task::spawn_blocking(move || ratatui::run(|terminal| SetupApp::new(handle, config).run(terminal)))
        .await
        .context("setup TUI task failed")??;
    Ok(())
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    #[test]
    fn welcome_screen_renders_key_promises() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let app = SetupApp::new(runtime.handle().clone(), Config::default());
        let backend = TestBackend::new(100, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.render(frame)).unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert!(rendered.contains("Your clipboard, without borders"));
        assert!(rendered.contains("No cloud account"));
        assert!(rendered.contains("No conversion"));
    }

    #[test]
    fn entry_screen_renders_verification_error_without_controls() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let mut app = SetupApp::new(runtime.handle().clone(), Config::default());
        app.stage = Stage::Entry;
        app.error = Some("bad\u{1b}[31m connection".into());
        let backend = TestBackend::new(100, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.render(frame)).unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert!(rendered.contains("Couldn’t verify"));
        assert!(!rendered.contains('\u{1b}'));
    }
}
