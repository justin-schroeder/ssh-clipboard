use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::process::Command;

use crate::config::{ensure_private_dir, paths};

const LABEL: &str = "dev.ssh-clipboard";

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Start,
    Stop,
    Restart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallOutcome {
    Running,
    PendingLogin,
}

impl InstallOutcome {
    #[must_use]
    pub const fn detail(self) -> &'static str {
        match self {
            Self::Running => "Installed and running",
            Self::PendingLogin => "Installed; starts at next login",
        }
    }

    #[must_use]
    pub fn from_detail(value: &str) -> Option<Self> {
        match value.trim() {
            "Installed and running" => Some(Self::Running),
            "Installed; starts at next login" => Some(Self::PendingLogin),
            _ => None,
        }
    }
}

pub async fn install(binary: &Path) -> Result<InstallOutcome> {
    if !binary.is_absolute() {
        bail!("service binary path must be absolute");
    }
    let expected_version = binary_version(binary).await?;
    let paths = paths()?;
    if let Some(parent) = paths.service.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    ensure_private_dir(&paths.state_dir)?;
    let contents = if cfg!(target_os = "macos") {
        launch_agent(binary, &paths.log)
    } else if cfg!(target_os = "linux") {
        systemd_unit(binary)
    } else {
        bail!("background services are supported on macOS and Linux");
    };
    write_atomic(&paths.service, contents.as_bytes()).await?;
    let outcome = if cfg!(target_os = "macos") {
        start_macos(&paths.service).await
    } else {
        start_linux().await
    }?;
    if outcome == InstallOutcome::Running {
        wait_until_healthy(&paths.socket, &expected_version).await?;
    }
    Ok(outcome)
}

pub async fn control(action: Action) -> Result<()> {
    if cfg!(target_os = "macos") {
        let domain = format!("gui/{}", uid().await?);
        match action {
            Action::Start => run("launchctl", &["kickstart", &format!("{domain}/{LABEL}")]).await,
            Action::Stop => run("launchctl", &["bootout", &format!("{domain}/{LABEL}")]).await,
            Action::Restart => run("launchctl", &["kickstart", "-k", &format!("{domain}/{LABEL}")]).await,
        }
    } else if cfg!(target_os = "linux") {
        let action = match action {
            Action::Start => "start",
            Action::Stop => "stop",
            Action::Restart => "restart",
        };
        run("systemctl", &["--user", action, "ssh-clipboard.service"]).await
    } else {
        bail!("background services are supported on macOS and Linux")
    }
}

fn launch_agent(binary: &Path, log: &Path) -> String {
    let binary = xml_escape(&binary.display().to_string());
    let log = xml_escape(&log.display().to_string());
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>{LABEL}</string>
  <key>ProgramArguments</key><array><string>{binary}</string><string>daemon</string></array>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>ProcessType</key><string>Interactive</string>
  <key>StandardOutPath</key><string>{log}</string>
  <key>StandardErrorPath</key><string>{log}</string>
</dict></plist>
"#
    )
}

fn systemd_unit(binary: &Path) -> String {
    let binary = systemd_quote(&binary.display().to_string());
    format!(
        r"[Unit]
Description=Native encrypted clipboard sync over SSH
After=graphical-session.target

[Service]
Type=simple
ExecStart={binary} daemon
Restart=always
RestartSec=1
PassEnvironment=DISPLAY WAYLAND_DISPLAY XDG_RUNTIME_DIR DBUS_SESSION_BUS_ADDRESS

[Install]
WantedBy=default.target
"
    )
}

async fn start_macos(service_path: &Path) -> Result<InstallOutcome> {
    let domain = format!("gui/{}", uid().await?);
    if !command_succeeds("launchctl", &["print", &domain]).await? {
        return Ok(InstallOutcome::PendingLogin);
    }

    let service = format!("{domain}/{LABEL}");
    if !command_succeeds("launchctl", &["print", &service]).await? {
        let bootstrap = run(
            "launchctl",
            &["bootstrap", &domain, &service_path.display().to_string()],
        )
        .await;
        if let Err(error) = bootstrap
            && !command_succeeds("launchctl", &["print", &service]).await?
        {
            return Err(error).context("install LaunchAgent");
        }
    }

    run("launchctl", &["kickstart", "-k", &service])
        .await
        .context("start LaunchAgent")?;
    Ok(InstallOutcome::Running)
}

async fn start_linux() -> Result<InstallOutcome> {
    if !command_succeeds("systemctl", &["--user", "show-environment"]).await? {
        return Ok(InstallOutcome::PendingLogin);
    }
    let _ = run(
        "systemctl",
        &[
            "--user",
            "import-environment",
            "DISPLAY",
            "WAYLAND_DISPLAY",
            "XDG_RUNTIME_DIR",
            "DBUS_SESSION_BUS_ADDRESS",
        ],
    )
    .await;
    run("systemctl", &["--user", "daemon-reload"]).await?;
    run(
        "systemctl",
        &["--user", "enable", "--now", "ssh-clipboard.service"],
    )
    .await?;
    Ok(InstallOutcome::Running)
}

async fn uid() -> Result<String> {
    let output = Command::new("id").arg("-u").output().await?;
    if !output.status.success() {
        bail!("could not determine user id");
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

async fn run(program: &str, arguments: &[&str]) -> Result<()> {
    let output = Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .output()
        .await
        .with_context(|| format!("run {program}"))?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    Ok(())
}

async fn command_succeeds(program: &str, arguments: &[&str]) -> Result<bool> {
    Ok(Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .with_context(|| format!("run {program}"))?
        .success())
}

async fn binary_version(binary: &Path) -> Result<String> {
    let output = Command::new(binary).arg("--version").output().await?;
    if !output.status.success() {
        bail!("service binary failed its version check");
    }
    let output = String::from_utf8(output.stdout)?;
    output
        .trim()
        .strip_prefix("ssh-clipboard ")
        .filter(|version| !version.is_empty())
        .map(str::to_owned)
        .context("service binary reported an invalid version")
}

async fn wait_until_healthy(socket: &Path, expected_version: &str) -> Result<()> {
    let mut last_error = None;
    for _ in 0..40 {
        match UnixStream::connect(socket).await {
            Ok(mut stream) => {
                if let Err(error) = stream.write_all(b"STATUS\n").await {
                    last_error = Some(error.into());
                } else {
                    let mut response = String::new();
                    match tokio::time::timeout(
                        Duration::from_millis(500),
                        BufReader::new(stream).read_line(&mut response),
                    )
                    .await
                    {
                        Ok(Ok(read))
                            if read > 0
                                && serde_json::from_str::<serde_json::Value>(&response).is_ok_and(
                                    |status| {
                                        status.get("running").and_then(serde_json::Value::as_bool)
                                            == Some(true)
                                            && status.get("version").and_then(serde_json::Value::as_str)
                                                == Some(expected_version)
                                    },
                                ) =>
                        {
                            return Ok(());
                        }
                        Ok(Ok(_)) => {
                            last_error =
                                Some(anyhow::anyhow!("daemon returned an empty or unhealthy status"));
                        }
                        Ok(Err(error)) => last_error = Some(error.into()),
                        Err(error) => last_error = Some(error.into()),
                    }
                }
            }
            Err(error) => last_error = Some(error.into()),
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("daemon did not create its status socket")))
        .context("background service did not become healthy")
}

async fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let temporary = PathBuf::from(format!("{}.new", path.display()));
    tokio::fs::write(&temporary, bytes).await?;
    tokio::fs::rename(temporary, path).await?;
    Ok(())
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn systemd_quote(value: &str) -> String {
    format!(r#""{}""#, value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_documents_escape_paths() {
        let plist = launch_agent(Path::new("/Users/a&b/tool"), Path::new("/tmp/a&b.log"));
        assert!(plist.contains("/Users/a&amp;b/tool"));
        let unit = systemd_unit(Path::new("/home/me/My Tools/tool"));
        assert!(unit.contains("ExecStart=\"/home/me/My Tools/tool\" daemon"));
    }

    #[test]
    fn install_outcomes_round_trip_through_remote_output() {
        for outcome in [InstallOutcome::Running, InstallOutcome::PendingLogin] {
            assert_eq!(InstallOutcome::from_detail(outcome.detail()), Some(outcome));
        }
    }

    #[tokio::test]
    async fn health_check_requires_a_live_status_response() {
        let directory = tempfile::tempdir().unwrap();
        let socket = directory.path().join("daemon.sock");
        let listener = tokio::net::UnixListener::bind(&socket).unwrap();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            stream
                .write_all(b"{\"running\":true,\"version\":\"test\"}\n")
                .await
                .unwrap();
        });

        wait_until_healthy(&socket, "test").await.unwrap();
    }
}
