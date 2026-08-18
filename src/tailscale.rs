use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use tokio::process::Command;
use tokio::time::timeout;

const STATUS_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Peer {
    pub hostname: String,
    pub dns_name: String,
    pub os: String,
}

impl Peer {
    #[must_use]
    pub fn ssh_command(&self) -> String {
        format!("ssh {}", self.dns_name)
    }
}

#[derive(Debug, Deserialize)]
struct Status {
    #[serde(rename = "Peer", default)]
    peers: HashMap<String, StatusPeer>,
}

#[derive(Debug, Deserialize)]
struct StatusPeer {
    #[serde(rename = "HostName", default)]
    hostname: String,
    #[serde(rename = "DNSName", default)]
    dns_name: String,
    #[serde(rename = "OS", default)]
    os: String,
    #[serde(rename = "Online", default)]
    online: bool,
    #[serde(rename = "TailscaleIPs", default)]
    addresses: Vec<String>,
}

pub async fn discover() -> Result<Vec<Peer>> {
    let output = if let Some(output) = try_status(Path::new("tailscale")).await? {
        output
    } else if let Some(executable) = fallback_cli() {
        try_status(&executable)
            .await?
            .context("Tailscale application CLI disappeared")?
    } else {
        return Ok(Vec::new());
    };
    if !output.status.success() {
        bail!(
            "Tailscale status failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    parse_status(&output.stdout)
}

async fn try_status(executable: &Path) -> Result<Option<Output>> {
    match timeout(STATUS_TIMEOUT, status_command(executable).output()).await {
        Ok(Ok(output)) => Ok(Some(output)),
        Ok(Err(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Ok(Err(error)) => Err(error).context("run Tailscale CLI"),
        Err(error) => Err(error).context("Tailscale status timed out"),
    }
}

fn fallback_cli() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let app_cli = PathBuf::from("/Applications/Tailscale.app/Contents/MacOS/Tailscale");
        if app_cli.is_file() {
            return Some(app_cli);
        }
    }
    None
}

fn status_command(executable: &Path) -> Command {
    let mut command = Command::new(executable);
    command.args(["status", "--json"]);
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.kill_on_drop(true);
    command
}

fn parse_status(bytes: &[u8]) -> Result<Vec<Peer>> {
    let status: Status = serde_json::from_slice(bytes).context("decode Tailscale status")?;
    let mut peers = status
        .peers
        .into_values()
        .filter(|peer| peer.online && compatible_os(&peer.os))
        .filter_map(|peer| {
            let dns_name = peer.dns_name.trim().trim_end_matches('.');
            let address = if dns_name.is_empty() {
                peer.addresses
                    .iter()
                    .find(|address| address.contains('.'))
                    .map(String::as_str)
            } else {
                Some(dns_name)
            }?;
            let hostname = if peer.hostname.trim().is_empty() {
                address.split('.').next().unwrap_or(address)
            } else {
                peer.hostname.trim()
            };
            Some(Peer {
                hostname: hostname.to_owned(),
                dns_name: address.to_owned(),
                os: display_os(&peer.os).to_owned(),
            })
        })
        .collect::<Vec<_>>();
    peers.sort_by_key(|peer| peer.hostname.to_ascii_lowercase());
    peers.dedup_by(|left, right| left.dns_name == right.dns_name);
    Ok(peers)
}

fn compatible_os(os: &str) -> bool {
    matches!(
        os.trim().to_ascii_lowercase().as_str(),
        "macos" | "darwin" | "linux"
    )
}

fn display_os(os: &str) -> &'static str {
    match os.trim().to_ascii_lowercase().as_str() {
        "macos" | "darwin" => "macOS",
        "linux" => "Linux",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_keeps_only_online_supported_machines() {
        let peers = parse_status(
            br#"{
                "Peer": {
                    "node-a": {
                        "HostName": "Studio Mac",
                        "DNSName": "studio.example.ts.net.",
                        "OS": "macOS",
                        "Online": true,
                        "TailscaleIPs": ["100.64.0.1"]
                    },
                    "node-b": {
                        "HostName": "server",
                        "DNSName": "server.example.ts.net.",
                        "OS": "linux",
                        "Online": true,
                        "TailscaleIPs": ["100.64.0.2"]
                    },
                    "node-c": {
                        "HostName": "phone",
                        "DNSName": "phone.example.ts.net.",
                        "OS": "iOS",
                        "Online": true
                    },
                    "node-d": {
                        "HostName": "sleeping",
                        "DNSName": "sleeping.example.ts.net.",
                        "OS": "linux",
                        "Online": false
                    }
                }
            }"#,
        )
        .unwrap();

        assert_eq!(
            peers,
            vec![
                Peer {
                    hostname: "server".into(),
                    dns_name: "server.example.ts.net".into(),
                    os: "Linux".into(),
                },
                Peer {
                    hostname: "Studio Mac".into(),
                    dns_name: "studio.example.ts.net".into(),
                    os: "macOS".into(),
                },
            ]
        );
    }

    #[test]
    fn status_falls_back_to_an_ipv4_address() {
        let peers = parse_status(
            br#"{"Peer":{"node":{"HostName":"server","OS":"linux","Online":true,"TailscaleIPs":["fd7a::1","100.64.0.8"]}}}"#,
        )
        .unwrap();

        assert_eq!(peers[0].ssh_command(), "ssh 100.64.0.8");
    }
}
