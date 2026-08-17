use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{Context, Result, bail};
use tokio::process::Command;

use crate::config::{ensure_private_dir, paths};

const LABEL: &str = "dev.ssh-clipboard";

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Start,
    Stop,
    Restart,
}

pub async fn install(binary: &Path) -> Result<()> {
    if !binary.is_absolute() {
        bail!("service binary path must be absolute");
    }
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
    if cfg!(target_os = "macos") {
        start_macos(&paths.service).await
    } else {
        start_linux().await
    }
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

async fn start_macos(service_path: &Path) -> Result<()> {
    let domain = format!("gui/{}", uid().await?);
    let _ = run("launchctl", &["bootout", &format!("{domain}/{LABEL}")]).await;
    run(
        "launchctl",
        &["bootstrap", &domain, &service_path.display().to_string()],
    )
    .await
    .context("install LaunchAgent; a logged-in GUI session is required")?;
    run("launchctl", &["kickstart", "-k", &format!("{domain}/{LABEL}")]).await
}

async fn start_linux() -> Result<()> {
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
    .await
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
}
