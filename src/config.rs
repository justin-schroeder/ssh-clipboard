use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const CONFIG_VERSION: u16 = 1;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerConfig {
    pub name: String,
    pub ssh_command: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub version: u16,
    pub node_id: Uuid,
    pub node_name: String,
    #[serde(default)]
    pub peers: Vec<PeerConfig>,
    pub max_bytes: u64,
    pub poll_interval_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        let node_name = detected_machine_name();
        Self {
            version: CONFIG_VERSION,
            node_id: Uuid::new_v4(),
            node_name,
            peers: Vec::new(),
            max_bytes: 256 * 1024 * 1024,
            poll_interval_ms: 75,
        }
    }
}

#[must_use]
pub fn detected_machine_name() -> String {
    hostname::get()
        .ok()
        .and_then(|hostname| hostname.into_string().ok())
        .filter(|hostname| !hostname.trim().is_empty())
        .unwrap_or_else(|| "unknown".to_owned())
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        if self.version != CONFIG_VERSION {
            bail!("unsupported config version {}", self.version);
        }
        if self.node_name.trim().is_empty() {
            bail!("node name cannot be empty");
        }
        if self.max_bytes == 0 {
            bail!("max_bytes must be positive");
        }
        if !(20..=5_000).contains(&self.poll_interval_ms) {
            bail!("poll_interval_ms must be between 20 and 5000");
        }
        for peer in &self.peers {
            if peer.name.trim().is_empty() || peer.ssh_command.trim().is_empty() {
                bail!("every peer requires a name and SSH command");
            }
        }
        Ok(())
    }

    pub fn load() -> Result<Self> {
        Self::load_at(&paths()?.config_file)
    }

    pub fn load_at(path: &Path) -> Result<Self> {
        let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
        let config: Self =
            serde_json::from_slice(&bytes).with_context(|| format!("decode {}", path.display()))?;
        config.validate()?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        self.save_at(&paths()?.config_file)
    }

    pub fn save_at(&self, path: &Path) -> Result<()> {
        self.validate()?;
        let parent = path.parent().context("config path has no parent")?;
        ensure_private_dir(parent).with_context(|| format!("create {}", parent.display()))?;
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        let temporary = path.with_extension("json.new");
        let mut file = fs::File::create(&temporary)?;
        set_private_permissions(&file)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)?;
        Ok(())
    }
}

pub fn ensure_private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct Paths {
    pub config_dir: PathBuf,
    pub config_file: PathBuf,
    pub state_dir: PathBuf,
    pub socket: PathBuf,
    pub log: PathBuf,
    pub binary: PathBuf,
    pub service: PathBuf,
}

pub fn paths() -> Result<Paths> {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .context("HOME is not set")?;
    let config_dir = env::var_os("SSH_CLIPBOARD_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| env::var_os("XDG_CONFIG_HOME").map(|base| PathBuf::from(base).join("ssh-clipboard")))
        .unwrap_or_else(|| home.join(".config/ssh-clipboard"));
    let state_dir = env::var_os("SSH_CLIPBOARD_STATE_DIR")
        .map(PathBuf::from)
        .or_else(|| env::var_os("XDG_STATE_HOME").map(|base| PathBuf::from(base).join("ssh-clipboard")))
        .unwrap_or_else(|| home.join(".local/state/ssh-clipboard"));
    let service = if cfg!(target_os = "macos") {
        home.join("Library/LaunchAgents/dev.ssh-clipboard.plist")
    } else {
        home.join(".config/systemd/user/ssh-clipboard.service")
    };
    Ok(Paths {
        config_file: config_dir.join("config.json"),
        config_dir,
        socket: state_dir.join("daemon.sock"),
        log: state_dir.join("daemon.log"),
        state_dir,
        binary: home.join(".local/bin/ssh-clipboard"),
        service,
    })
}

#[cfg(unix)]
fn set_private_permissions(file: &fs::File) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_private_permissions(_file: &fs::File) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_round_trip_and_permissions() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("nested/config.json");
        let config = Config::default();
        config.save_at(&path).unwrap();
        assert_eq!(Config::load_at(&path).unwrap(), config);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(path).unwrap().permissions().mode() & 0o777, 0o600);
            assert_eq!(
                fs::metadata(directory.path().join("nested"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o700
            );
        }
    }

    #[test]
    fn invalid_poll_interval_is_rejected() {
        let config = Config {
            poll_interval_ms: 5,
            ..Config::default()
        };
        assert!(config.validate().is_err());
    }
}
