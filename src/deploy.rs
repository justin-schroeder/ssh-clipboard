use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::config::{Config, paths};
use crate::service;
use crate::ssh::{self, ProbeResult};

pub fn binary_for(os: &str, arch: &str) -> Result<PathBuf> {
    if !matches!(os, "darwin" | "linux") || !matches!(arch, "arm64" | "amd64") {
        bail!("unsupported target {os}/{arch}");
    }
    let current = std::env::current_exe()?;
    if current_target() == (os, arch) {
        return Ok(current);
    }
    let filename = format!("ssh-clipboard-{os}-{arch}");
    let bundle_root = std::env::var_os("SSH_CLIPBOARD_BINARIES_DIR").map(PathBuf::from);
    let candidates = binary_candidates(&current, os, arch, bundle_root.as_deref());
    candidates
        .into_iter()
        .find(|candidate| candidate.is_file())
        .with_context(|| {
            format!(
                "this installation does not include a {os}/{arch} peer binary; use a release bundle containing {filename}"
            )
        })
}

fn binary_candidates(current: &Path, os: &str, arch: &str, bundle_root: Option<&Path>) -> Vec<PathBuf> {
    let filename = format!("ssh-clipboard-{os}-{arch}");
    let mut candidates = Vec::with_capacity(5);
    if let Some(root) = bundle_root {
        candidates.push(root.join(format!("{os}-{arch}")).join("ssh-clipboard"));
        candidates.push(root.join(&filename));
    }
    let executable_dir = current.parent().unwrap_or_else(|| Path::new("."));
    candidates.push(executable_dir.join(&filename));
    candidates.push(executable_dir.join("dist").join(&filename));
    candidates.push(PathBuf::from("dist").join(filename));
    candidates
}

pub async fn install_remote<F>(ssh_command: &str, probe: &ProbeResult, mut progress: F) -> Result<()>
where
    F: FnMut(&str, &str),
{
    progress("prepare", "Selecting the peer binary");
    let binary = binary_for(&probe.os, &probe.arch)?;
    progress("upload", "Streaming the binary over encrypted SSH");
    ssh::upload_binary(ssh_command, &binary).await?;
    let mut remote = Config::default();
    if !probe.hostname.is_empty() {
        remote.node_name.clone_from(&probe.hostname);
    }
    let mut encoded = serde_json::to_vec_pretty(&remote)?;
    encoded.push(b'\n');
    progress("configure", "Writing private node configuration");
    ssh::upload_config(ssh_command, &encoded).await?;
    progress("service", "Installing the per-user background service");
    ssh::run(
        ssh_command,
        r#"exec "$HOME/.local/bin/ssh-clipboard" service install --binary "$HOME/.local/bin/ssh-clipboard""#,
    )
    .await?;
    Ok(())
}

pub async fn install_local_binary(source: Option<&Path>) -> Result<PathBuf> {
    let source = match source {
        Some(source) => source.to_path_buf(),
        None => std::env::current_exe()?,
    };
    let destination = paths()?.binary;
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let temporary = PathBuf::from(format!("{}.new", destination.display()));
    tokio::fs::copy(source, &temporary).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(&temporary, std::fs::Permissions::from_mode(0o755)).await?;
    }
    if destination.is_file() {
        let previous = PathBuf::from(format!("{}.previous", destination.display()));
        tokio::fs::copy(&destination, &previous).await?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(&previous, std::fs::Permissions::from_mode(0o700)).await?;
        }
    }
    tokio::fs::rename(&temporary, &destination).await?;
    Ok(destination)
}

pub async fn install_local_service() -> Result<()> {
    let binary = install_local_binary(None).await?;
    service::install(&binary).await
}

pub async fn restore_previous_binary() -> Result<PathBuf> {
    let destination = paths()?.binary;
    let previous = PathBuf::from(format!("{}.previous", destination.display()));
    if !previous.is_file() {
        bail!("no previous ssh-clipboard binary is available");
    }
    let temporary = PathBuf::from(format!("{}.rollback", destination.display()));
    tokio::fs::copy(&previous, &temporary).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(&temporary, std::fs::Permissions::from_mode(0o755)).await?;
    }
    tokio::fs::rename(temporary, &destination).await?;
    Ok(destination)
}

fn current_target() -> (&'static str, &'static str) {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "x86_64" => "amd64",
        other => other,
    };
    (os, arch)
}

pub fn current_target_name() -> Result<String> {
    let (os, arch) = current_target();
    if !matches!(os, "darwin" | "linux") || !matches!(arch, "arm64" | "amd64") {
        bail!("unsupported current target {os}/{arch}");
    }
    Ok(format!("{os}-{arch}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_binary_is_selected_for_the_current_target() {
        let (os, arch) = current_target();
        assert_eq!(binary_for(os, arch).unwrap(), std::env::current_exe().unwrap());
    }

    #[test]
    fn unsupported_targets_are_rejected() {
        assert!(binary_for("plan9", "mips").is_err());
    }

    #[test]
    fn npm_bundle_layout_is_searched_first() {
        let candidates = binary_candidates(
            Path::new("/app/vendor/darwin-arm64/ssh-clipboard"),
            "linux",
            "amd64",
            Some(Path::new("/app/vendor")),
        );
        assert_eq!(
            candidates[0],
            PathBuf::from("/app/vendor/linux-amd64/ssh-clipboard")
        );
        assert_eq!(
            candidates[1],
            PathBuf::from("/app/vendor/ssh-clipboard-linux-amd64")
        );
    }
}
