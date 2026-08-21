use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use ssh_clipboard::config::{Config, paths};
use ssh_clipboard::daemon;
use ssh_clipboard::model::{Direction, MonitorEvent, human_bytes};
use ssh_clipboard::{deploy, service, tui, update};
use tokio::io::AsyncBufReadExt;

const RUNTIME_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Parser)]
#[command(
    name = "ssh-clipboard",
    version,
    about = "Native clipboard sync over encrypted SSH",
    long_about = "Copy normally on one macOS or Linux machine and paste on another. Original clipboard formats travel over persistent passwordless SSH connections."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Configure, verify, and install peers
    Setup,
    /// Watch clipboard values and peer health
    Monitor {
        /// Stream readable lines instead of the TUI
        #[arg(long, conflicts_with = "json")]
        plain: bool,
        /// Stream newline-delimited JSON instead of the TUI
        #[arg(long)]
        json: bool,
    },
    /// Show daemon and connection status
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Check for and install the latest stable release
    Update {
        /// Report versions without installing
        #[arg(long)]
        check: bool,
    },
    /// Manage the per-user background service
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
    #[command(hide = true)]
    Daemon,
    #[command(hide = true)]
    Bridge,
    #[command(hide = true)]
    UpdateWatchdog { version: String },
}

#[derive(Subcommand)]
enum ServiceAction {
    Install {
        #[arg(long)]
        binary: Option<PathBuf>,
    },
    Start,
    Stop,
    Restart,
}

fn main() {
    let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("ssh-clipboard: initialize async runtime: {error}");
            std::process::exit(1);
        }
    };
    let result = runtime.block_on(run());
    shutdown_runtime(runtime);
    if let Err(error) = result {
        eprintln!("ssh-clipboard: {error:#}");
        std::process::exit(1);
    }
}

fn shutdown_runtime(runtime: tokio::runtime::Runtime) {
    runtime.shutdown_timeout(RUNTIME_SHUTDOWN_TIMEOUT);
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        None => {
            let config_path = paths()?.config_file;
            if config_path.exists() {
                tui::run_monitor(Config::load()?).await
            } else {
                tui::run_setup(Config::default()).await
            }
        }
        Some(Command::Setup) => {
            let config = if paths()?.config_file.exists() {
                Config::load()?
            } else {
                Config::default()
            };
            tui::run_setup(config).await
        }
        Some(Command::Monitor { plain, json }) => {
            if plain || json {
                stream_monitor(json).await
            } else {
                tui::run_monitor(Config::load()?).await
            }
        }
        Some(Command::Status { json }) => {
            let status = daemon::query_status().await.context("daemon is not running")?;
            if json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else {
                println!(
                    "running as {} ({}, {}, version {})",
                    if status.machine_name.is_empty() {
                        &status.node_name
                    } else {
                        &status.machine_name
                    },
                    status.node_id,
                    status.clipboard_backend,
                    status.version
                );
                if status.desired_version != status.version {
                    println!("updating to: {}", status.desired_version);
                }
                if status.connected_peers.is_empty() {
                    println!("no peers connected");
                } else {
                    for peer in status.connected_peers {
                        println!("connected: {peer}");
                    }
                }
                for peer in status.peers {
                    let version = peer
                        .version
                        .as_deref()
                        .filter(|version| *version != "legacy" && !version.is_empty())
                        .unwrap_or("unknown");
                    println!("peer version: {} {}", peer.name, version);
                }
            }
            Ok(())
        }
        Some(Command::Update { check }) => {
            if check {
                let latest = update::latest_version().await?;
                println!("current: {}", update::CURRENT_VERSION);
                println!("latest:  {latest}");
                return Ok(());
            }
            if let Some(version) = update::update_now().await? {
                println!("installed {version}; reconciling service");
            } else {
                println!("no newer stable release than {}", update::CURRENT_VERSION);
            }
            let outcome = deploy::install_local_service().await?;
            println!("{}", outcome.detail());
            Ok(())
        }
        Some(Command::Service { action }) => match action {
            ServiceAction::Install { binary } => {
                let binary = match binary {
                    Some(binary) => binary,
                    None => std::env::current_exe()?,
                };
                let binary = binary.canonicalize().context("resolve service binary")?;
                let outcome = service::install(&binary).await?;
                println!("{}", outcome.detail());
                Ok(())
            }
            ServiceAction::Start => service::control(service::Action::Start).await,
            ServiceAction::Stop => service::control(service::Action::Stop).await,
            ServiceAction::Restart => service::control(service::Action::Restart).await,
        },
        Some(Command::Daemon) => {
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "ssh_clipboard=info".into()),
                )
                .with_writer(std::io::stderr)
                .init();
            daemon::run(Config::load()?).await
        }
        Some(Command::Bridge) => daemon::bridge().await,
        Some(Command::UpdateWatchdog { version }) => update::watchdog(&version).await,
    }
}

async fn stream_monitor(json: bool) -> Result<()> {
    let mut reader = daemon::connect_monitor().await.context("daemon is not running")?;
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            bail!("daemon monitor stream closed");
        }
        let event: MonitorEvent = serde_json::from_str(&line)?;
        if json {
            print!("{line}");
            continue;
        }
        let flow = match event.direction {
            Direction::Local => "local".to_owned(),
            Direction::Send => format!("send → {}", event.peer.as_deref().unwrap_or("peer")),
            Direction::Receive => format!("recv ← {}", event.peer.as_deref().unwrap_or("peer")),
        };
        println!(
            "{}  {:<24} {:<10} {:>2} formats  {}",
            format_time(event.timestamp_millis),
            flow,
            human_bytes(event.total_bytes()),
            event.representations.len(),
            event.preview
        );
    }
}

fn format_time(timestamp_millis: u64) -> String {
    let day = timestamp_millis % 86_400_000;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        day / 3_600_000,
        (day / 60_000) % 60,
        (day / 1_000) % 60,
        day % 1_000
    )
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Instant;

    use super::*;

    #[test]
    fn runtime_shutdown_is_bounded_when_blocking_work_hangs() {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let (started_tx, started_rx) = mpsc::sync_channel(0);
        let (release_tx, release_rx) = mpsc::channel();
        let (finished_tx, finished_rx) = mpsc::sync_channel(0);
        let _task = runtime.handle().spawn_blocking(move || {
            started_tx.send(()).unwrap();
            let _ = release_rx.recv();
            finished_tx.send(()).unwrap();
        });
        started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let started = Instant::now();

        shutdown_runtime(runtime);

        assert!(started.elapsed() < Duration::from_millis(1_500));
        release_tx.send(()).unwrap();
        finished_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    }
}
