use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};
use tokio::time::timeout;
use uuid::Uuid;

const SSH_OPTIONS: &[&str] = &[
    "-T",
    "-oBatchMode=yes",
    "-oPasswordAuthentication=no",
    "-oKbdInteractiveAuthentication=no",
    "-oStrictHostKeyChecking=accept-new",
    "-oConnectTimeout=10",
    "-oServerAliveInterval=15",
    "-oServerAliveCountMax=3",
    "-oClearAllForwardings=yes",
    "-oLogLevel=ERROR",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProbeResult {
    pub os: String,
    pub arch: String,
    pub home: String,
    pub hostname: String,
}

#[must_use]
pub fn normalize_command(raw: &str) -> String {
    let trimmed = raw.trim();
    let is_ssh_command = shell_words::split(trimmed)
        .ok()
        .and_then(|words| words.into_iter().next())
        .is_some_and(|program| Path::new(&program).file_name().and_then(|name| name.to_str()) == Some("ssh"));
    if is_ssh_command {
        trimmed.to_owned()
    } else {
        format!("ssh {trimmed}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedCommand {
    program: PathBuf,
    arguments: Vec<String>,
}

pub async fn probe(raw: &str) -> Result<ProbeResult> {
    let token = format!("SCB_{}", Uuid::new_v4().simple());
    let remote =
        format!("printf '{token}\\t'; uname -s; uname -m; uname -n; printf '{token}\\t%s\\n' \"$HOME\"");
    let output = timeout(Duration::from_secs(20), command(raw, &remote)?.output())
        .await
        .context("SSH verification timed out")??;
    if !output.status.success() {
        bail!(
            "passwordless SSH verification failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    parse_probe(&String::from_utf8_lossy(&output.stdout), &token)
}

pub fn start_bridge(raw: &str) -> Result<Child> {
    let mut command = command(raw, r#"exec "$HOME/.local/bin/ssh-clipboard" bridge"#)?;
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    command.kill_on_drop(true);
    command.spawn().context("start persistent SSH bridge")
}

pub async fn upload_binary(raw: &str, binary: &Path) -> Result<()> {
    let remote = r#"set -eu; umask 077; mkdir -p "$HOME/.local/bin"; tmp="$HOME/.local/bin/.ssh-clipboard.tmp.$$"; trap 'rm -f "$tmp"' EXIT; cat > "$tmp"; chmod 755 "$tmp"; mv "$tmp" "$HOME/.local/bin/ssh-clipboard"; trap - EXIT"#;
    upload(raw, remote, &tokio::fs::read(binary).await?).await
}

pub async fn upload_config(raw: &str, bytes: &[u8]) -> Result<()> {
    let remote = r#"set -eu; umask 077; mkdir -p "$HOME/.config/ssh-clipboard"; tmp="$HOME/.config/ssh-clipboard/.config.tmp.$$"; trap 'rm -f "$tmp"' EXIT; cat > "$tmp"; chmod 600 "$tmp"; mv "$tmp" "$HOME/.config/ssh-clipboard/config.json"; trap - EXIT"#;
    upload(raw, remote, bytes).await
}

pub async fn run(raw: &str, remote: &str) -> Result<Vec<u8>> {
    let output = command(raw, remote)?.output().await?;
    if !output.status.success() {
        bail!(
            "remote command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(output.stdout)
}

async fn upload(raw: &str, remote: &str, bytes: &[u8]) -> Result<()> {
    let mut command = command(raw, remote)?;
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    command.kill_on_drop(true);
    let mut child = command.spawn()?;
    child
        .stdin
        .take()
        .context("SSH upload stdin unavailable")?
        .write_all(bytes)
        .await?;
    let output = child.wait_with_output().await?;
    if !output.status.success() {
        bail!(
            "SSH upload failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

fn command(raw: &str, remote: &str) -> Result<Command> {
    let parsed = parse_command(raw)?;
    let mut command = Command::new(parsed.program);
    command.args(SSH_OPTIONS).args(parsed.arguments).arg(remote);
    command.stdin(Stdio::null());
    command.kill_on_drop(true);
    Ok(command)
}

fn parse_command(raw: &str) -> Result<ParsedCommand> {
    let words = shell_words::split(raw).context("parse SSH command")?;
    let Some(program) = words.first() else {
        bail!("SSH command is empty");
    };
    if Path::new(program).file_name().and_then(|name| name.to_str()) != Some("ssh") {
        bail!("peer command must begin with ssh");
    }
    let mut found_destination = false;
    let mut index = 1;
    while index < words.len() {
        let word = &words[index];
        if found_destination {
            bail!("SSH command must not include a remote command");
        }
        if word == "--" {
            index += 1;
            if index >= words.len() {
                bail!("SSH command is missing a destination");
            }
            found_destination = true;
        } else if word.starts_with('-') {
            if option_takes_value(word) && word.len() == 2 {
                index += 1;
                if index >= words.len() {
                    bail!("SSH option {word} is missing a value");
                }
            }
        } else {
            found_destination = true;
        }
        index += 1;
    }
    if !found_destination {
        bail!("SSH command is missing a destination");
    }
    Ok(ParsedCommand {
        program: PathBuf::from(program),
        arguments: words[1..].to_vec(),
    })
}

fn option_takes_value(option: &str) -> bool {
    matches!(
        option,
        "-B" | "-b"
            | "-c"
            | "-D"
            | "-E"
            | "-e"
            | "-F"
            | "-I"
            | "-i"
            | "-J"
            | "-L"
            | "-l"
            | "-m"
            | "-O"
            | "-o"
            | "-P"
            | "-p"
            | "-Q"
            | "-R"
            | "-S"
            | "-W"
            | "-w"
    )
}

fn parse_probe(output: &str, token: &str) -> Result<ProbeResult> {
    let mut lines = output.lines();
    while let Some(line) = lines.next() {
        let Some(os) = line.strip_prefix(&format!("{token}\t")) else {
            continue;
        };
        let arch = lines.next().context("probe response is missing architecture")?;
        let hostname = lines.next().context("probe response is missing hostname")?;
        let home = lines
            .next()
            .and_then(|line| line.strip_prefix(&format!("{token}\t")))
            .context("probe response is missing home directory")?;
        return Ok(ProbeResult {
            os: normalize_os(os),
            arch: normalize_arch(arch),
            home: home.to_owned(),
            hostname: hostname.trim().to_owned(),
        });
    }
    bail!("SSH connected, but its probe response was invalid")
}

fn normalize_os(os: &str) -> String {
    match os.trim().to_ascii_lowercase().as_str() {
        "darwin" => "darwin".to_owned(),
        "linux" => "linux".to_owned(),
        other => other.to_owned(),
    }
}

fn normalize_arch(arch: &str) -> String {
    match arch.trim().to_ascii_lowercase().as_str() {
        "x86_64" | "amd64" => "amd64".to_owned(),
        "aarch64" | "arm64" => "arm64".to_owned(),
        other => other.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_connections_accept_only_new_host_keys() {
        assert!(SSH_OPTIONS.contains(&"-oStrictHostKeyChecking=accept-new"));
    }

    #[test]
    fn accepts_options_and_quoted_paths() {
        let parsed = parse_command(r#"ssh -i "/Users/me/My Key" -p 2222 person@example.com"#).unwrap();
        assert_eq!(parsed.program, PathBuf::from("ssh"));
        assert_eq!(parsed.arguments.last().unwrap(), "person@example.com");
    }

    #[test]
    fn normalizes_hosts_and_preserves_full_commands() {
        assert_eq!(normalize_command("macbookserver"), "ssh macbookserver");
        assert_eq!(normalize_command(" user@example.com "), "ssh user@example.com");
        assert_eq!(normalize_command("ssh -p 2222 server"), "ssh -p 2222 server");
        assert_eq!(normalize_command("/usr/bin/ssh server"), "/usr/bin/ssh server");
    }

    #[test]
    fn rejects_non_ssh_and_embedded_remote_commands() {
        assert!(parse_command("bash host").is_err());
        assert!(parse_command("ssh host uname -a").is_err());
    }

    #[test]
    fn parses_probe_with_banner_noise() {
        let output = "Welcome\nTOKEN\tDarwin\narm64\nmy-mac\nTOKEN\t/Users/me\n";
        assert_eq!(
            parse_probe(output, "TOKEN").unwrap(),
            ProbeResult {
                os: "darwin".into(),
                arch: "arm64".into(),
                home: "/Users/me".into(),
                hostname: "my-mac".into(),
            }
        );
    }
}
