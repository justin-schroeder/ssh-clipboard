use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{Mutex, RwLock, broadcast, mpsc, watch};
use tokio::task::JoinSet;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::clipboard::{ClipboardBackend, NativeClipboard};
use crate::config::{Config, PeerConfig, ensure_private_dir, paths};
use crate::filebundle;
use crate::model::{Clip, Direction, MonitorEvent};
use crate::protocol::{Message, read_message, write_clip, write_message};
use crate::ssh;
use crate::update::{self, CURRENT_VERSION};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerStatus {
    pub node_id: Uuid,
    pub name: String,
    pub version: Option<String>,
    pub desired_version: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Status {
    pub running: bool,
    pub node_id: Uuid,
    pub node_name: String,
    pub clipboard_backend: String,
    #[serde(default = "legacy_version")]
    pub version: String,
    #[serde(default = "legacy_version")]
    pub desired_version: String,
    #[serde(default)]
    pub configured_peers: Vec<String>,
    pub connected_peers: Vec<String>,
    #[serde(default)]
    pub peers: Vec<PeerStatus>,
}

fn legacy_version() -> String {
    "legacy".to_owned()
}

struct PeerLink {
    node_id: Uuid,
    name: String,
    version: Option<String>,
    desired_version: Option<String>,
    send: watch::Sender<Option<Arc<Clip>>>,
}

struct Daemon {
    config: Config,
    clipboard: Arc<dyn ClipboardBackend>,
    peers: RwLock<HashMap<Uuid, PeerLink>>,
    seen: Mutex<HashMap<Uuid, Instant>>,
    suppressed: Mutex<HashMap<[u8; 32], usize>>,
    apply_lock: Mutex<()>,
    events: broadcast::Sender<MonitorEvent>,
    desired_version: watch::Sender<String>,
    update_hints: mpsc::UnboundedSender<String>,
}

impl Daemon {
    #[cfg(test)]
    fn new(config: Config, clipboard: Arc<dyn ClipboardBackend>) -> Arc<Self> {
        let (desired_version, _) = watch::channel(CURRENT_VERSION.to_owned());
        let (update_hints, _) = mpsc::unbounded_channel();
        Self::with_updates(config, clipboard, desired_version, update_hints)
    }

    fn with_updates(
        config: Config,
        clipboard: Arc<dyn ClipboardBackend>,
        desired_version: watch::Sender<String>,
        update_hints: mpsc::UnboundedSender<String>,
    ) -> Arc<Self> {
        let (events, _) = broadcast::channel(256);
        Arc::new(Self {
            config,
            clipboard,
            peers: RwLock::new(HashMap::new()),
            seen: Mutex::new(HashMap::new()),
            suppressed: Mutex::new(HashMap::new()),
            apply_lock: Mutex::new(()),
            events,
            desired_version,
            update_hints,
        })
    }

    async fn watch_clipboard(self: Arc<Self>, mut shutdown: watch::Receiver<bool>) {
        let mut previous = self.clipboard.capture().await.ok().flatten();
        let poll = Duration::from_millis(self.config.poll_interval_ms);
        let mut changes = self.clipboard.change_receiver(poll);
        let mut interval = tokio::time::interval(Duration::from_millis(self.config.poll_interval_ms));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                () = next_clipboard_change(&mut changes, &mut interval) => {
                    let _guard = self.apply_lock.lock().await;
                    let snapshot = match self.clipboard.capture().await {
                        Ok(Some(snapshot)) => snapshot,
                        Ok(None) => continue,
                        Err(error) => {
                            debug!(%error, "clipboard capture skipped");
                            continue;
                        }
                    };
                    if previous.as_ref().is_some_and(|last| last.fingerprint == snapshot.fingerprint) {
                        continue;
                    }
                    previous = Some(snapshot.clone());
                    let mut suppressed = self.suppressed.lock().await;
                    if let Some(count) = suppressed.get_mut(&snapshot.fingerprint) {
                        *count -= 1;
                        if *count == 0 {
                            suppressed.remove(&snapshot.fingerprint);
                        }
                        continue;
                    }
                    drop(suppressed);
                    let clip = Arc::new(Clip::new(self.config.node_id, snapshot.representations));
                    self.mark_seen(clip.id).await;
                    self.emit(Direction::Local, None, &clip);
                    self.broadcast_clip(clip, None).await;
                }
                result = shutdown.changed() => {
                    if result.is_err() || *shutdown.borrow() {
                        return;
                    }
                }
            }
        }
    }

    async fn serve_peer<R, W>(
        self: Arc<Self>,
        reader: &mut R,
        writer: &mut W,
        label: &str,
        mut shutdown: watch::Receiver<bool>,
        established: Option<&AtomicBool>,
    ) -> Result<()>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let desired_version = self.desired_version.borrow().clone();
        write_message(
            writer,
            &Message::Hello {
                node_id: self.config.node_id,
                node_name: self.config.node_name.clone(),
                app_version: Some(CURRENT_VERSION.to_owned()),
                desired_version: Some(desired_version),
            },
            self.config.max_bytes,
        )
        .await?;
        let Message::Hello {
            node_id,
            node_name,
            app_version,
            desired_version,
        } = read_message(reader, self.config.max_bytes).await?
        else {
            bail!("peer did not begin with a hello message");
        };
        if let Some(version) = desired_version.as_deref()
            && update::newer_version(CURRENT_VERSION, version)
        {
            let _ = self.update_hints.send(version.to_owned());
        }
        let connection_id = Uuid::new_v4();
        let (sender, mut receiver) = watch::channel(None);
        let mut desired_updates = self.desired_version.subscribe();
        self.peers.write().await.insert(
            connection_id,
            PeerLink {
                node_id,
                name: node_name.clone(),
                version: app_version.clone(),
                desired_version: desired_version.clone(),
                send: sender,
            },
        );
        info!(peer = %node_name, %node_id, version = ?app_version, %label, "peer connected");
        if let Some(established) = established {
            established.store(true, Ordering::Release);
        }

        let result = loop {
            tokio::select! {
                incoming = read_message(reader, self.config.max_bytes) => {
                    match incoming {
                        Ok(Message::Clip(clip)) => self.receive_clip(Arc::new(clip), &node_name, connection_id).await,
                        Ok(Message::Hello { .. }) => {
                            break Err(anyhow::anyhow!("peer sent a second hello"));
                        }
                        Ok(Message::UpdateAvailable { version, .. }) => {
                            if let Some(peer) = self.peers.write().await.get_mut(&connection_id) {
                                peer.desired_version = Some(version.clone());
                            }
                            if update::newer_version(CURRENT_VERSION, &version) {
                                let _ = self.update_hints.send(version);
                            }
                        }
                        Err(error) => break Err(error.into()),
                    }
                }
                changed = receiver.changed() => {
                    if changed.is_err() {
                        break Ok(());
                    }
                    let clip = receiver.borrow_and_update().clone();
                    if let Some(clip) = clip {
                        if let Err(error) = write_clip(writer, &clip, self.config.max_bytes).await {
                            break Err(error.into());
                        }
                        self.emit(Direction::Send, Some(node_name.clone()), &clip);
                    }
                }
                changed = desired_updates.changed(), if app_version.is_some() => {
                    if changed.is_err() {
                        break Ok(());
                    }
                    let version = desired_updates.borrow_and_update().clone();
                    if let Err(error) = write_message(
                        writer,
                        &Message::UpdateAvailable {
                            update_id: Uuid::new_v4(),
                            version,
                        },
                        self.config.max_bytes,
                    ).await {
                        break Err(error.into());
                    }
                }
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow() {
                        break Ok(());
                    }
                }
            }
        };
        self.peers.write().await.remove(&connection_id);
        info!(peer = %node_name, "peer disconnected");
        result
    }

    async fn receive_clip(&self, clip: Arc<Clip>, peer_name: &str, source: Uuid) {
        if !self.mark_seen(clip.id).await {
            return;
        }
        self.emit(Direction::Receive, Some(peer_name.to_owned()), &clip);
        self.broadcast_clip(Arc::clone(&clip), Some(source)).await;
        let _guard = self.apply_lock.lock().await;
        let representations = match filebundle::materialize(clip.id, &clip.representations) {
            Ok(representations) => representations,
            Err(error) => {
                warn!(%error, peer = %peer_name, clip = %clip.id, "failed to materialize copied files");
                return;
            }
        };
        match self.clipboard.apply(&representations).await {
            Ok(snapshot) => {
                *self
                    .suppressed
                    .lock()
                    .await
                    .entry(snapshot.fingerprint)
                    .or_default() += 1;
            }
            Err(error) => warn!(%error, peer = %peer_name, clip = %clip.id, "failed to apply clipboard"),
        }
    }

    async fn broadcast_clip(&self, clip: Arc<Clip>, except: Option<Uuid>) {
        let peers = self.peers.read().await;
        for (id, peer) in peers.iter() {
            if Some(*id) != except {
                peer.send.send_replace(Some(Arc::clone(&clip)));
            }
        }
    }

    async fn mark_seen(&self, id: Uuid) -> bool {
        let mut seen = self.seen.lock().await;
        if seen.contains_key(&id) {
            return false;
        }
        seen.insert(id, Instant::now());
        if seen.len() > 4096 {
            let cutoff = Instant::now()
                .checked_sub(Duration::from_secs(3600))
                .unwrap_or_else(Instant::now);
            seen.retain(|_, instant| *instant >= cutoff);
        }
        true
    }

    fn emit(&self, direction: Direction, peer: Option<String>, clip: &Clip) {
        let _ = self.events.send(MonitorEvent::from_clip(direction, peer, clip));
    }

    async fn status(&self) -> Status {
        let peers = self.peers.read().await;
        let mut configured_peers = self
            .config
            .peers
            .iter()
            .map(|peer| peer.name.clone())
            .collect::<Vec<_>>();
        configured_peers.sort();
        configured_peers.dedup();
        let mut connected_peers = peers.values().map(|peer| peer.name.clone()).collect::<Vec<_>>();
        connected_peers.sort();
        connected_peers.dedup();
        let mut peer_statuses = peers
            .values()
            .map(|peer| PeerStatus {
                node_id: peer.node_id,
                name: peer.name.clone(),
                version: peer.version.clone(),
                desired_version: peer.desired_version.clone(),
            })
            .collect::<Vec<_>>();
        peer_statuses.sort_by(|left, right| left.name.cmp(&right.name));
        peer_statuses.dedup_by(|left, right| left.node_id == right.node_id);
        Status {
            running: true,
            node_id: self.config.node_id,
            node_name: self.config.node_name.clone(),
            clipboard_backend: self.clipboard.name().to_owned(),
            version: CURRENT_VERSION.to_owned(),
            desired_version: self.desired_version.borrow().clone(),
            configured_peers,
            connected_peers,
            peers: peer_statuses,
        }
    }
}

async fn next_clipboard_change(
    changes: &mut Option<tokio::sync::mpsc::UnboundedReceiver<()>>,
    interval: &mut tokio::time::Interval,
) {
    if let Some(receiver) = changes {
        if receiver.recv().await.is_some() {
            return;
        }
        *changes = None;
    }
    interval.tick().await;
}

pub async fn run(config: Config) -> Result<()> {
    let clipboard = Arc::new(NativeClipboard::new(config.max_bytes)?);
    let paths = paths()?;
    let (desired_version, _) = watch::channel(update::initial_desired_version());
    let (update_hints, hint_receiver) = mpsc::unbounded_channel();
    let update_desired = desired_version.clone();
    let shutdown = async move {
        tokio::select! {
            () = shutdown_signal() => {}
            version = update::run_auto_updates(update_desired, hint_receiver) => {
                info!(%version, "automatic update installed; requesting an explicit service restart");
                if let Err(error) = crate::service::control(crate::service::Action::Restart).await {
                    warn!(%error, %version, "explicit service restart failed; falling back to a clean daemon exit");
                }
            }
        }
    };
    run_daemon(
        config,
        clipboard,
        paths.socket,
        shutdown,
        desired_version,
        update_hints,
    )
    .await
}

async fn run_daemon<F>(
    config: Config,
    clipboard: Arc<dyn ClipboardBackend>,
    socket: PathBuf,
    shutdown: F,
    desired_version: watch::Sender<String>,
    update_hints: mpsc::UnboundedSender<String>,
) -> Result<()>
where
    F: Future<Output = ()>,
{
    if let Some(parent) = socket.parent() {
        ensure_private_dir(parent)?;
    }
    remove_stale_socket(&socket)?;
    let listener = UnixListener::bind(&socket).with_context(|| format!("bind {}", socket.display()))?;
    set_socket_permissions(&socket)?;
    update::mark_healthy().await?;
    let daemon = Daemon::with_updates(config, clipboard, desired_version, update_hints);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let mut tasks = JoinSet::new();
    tasks.spawn(Arc::clone(&daemon).watch_clipboard(shutdown_rx.clone()));
    tasks.spawn(accept_loop(Arc::clone(&daemon), listener, shutdown_rx.clone()));
    for peer in daemon.config.peers.clone() {
        tasks.spawn(dial_loop(Arc::clone(&daemon), peer, shutdown_rx.clone()));
    }
    info!(socket = %socket.display(), backend = daemon.clipboard.name(), "daemon ready");
    shutdown.await;
    let _ = shutdown_tx.send(true);
    tokio::time::sleep(Duration::from_millis(50)).await;
    tasks.abort_all();
    while tasks.join_next().await.is_some() {}
    let _ = tokio::fs::remove_file(&socket).await;
    Ok(())
}

async fn accept_loop(daemon: Arc<Daemon>, listener: UnixListener, mut shutdown: watch::Receiver<bool>) {
    loop {
        tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => {
                    let daemon = Arc::clone(&daemon);
                    let shutdown = shutdown.clone();
                    tokio::spawn(async move {
                        if let Err(error) = handle_local(daemon, stream, shutdown).await {
                            debug!(%error, "local socket closed");
                        }
                    });
                }
                Err(error) => {
                    warn!(%error, "local socket accept failed");
                    return;
                }
            },
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    return;
                }
            }
        }
    }
}

async fn handle_local(
    daemon: Arc<Daemon>,
    stream: UnixStream,
    shutdown: watch::Receiver<bool>,
) -> Result<()> {
    let mut buffered = BufReader::new(stream);
    let mut command = String::new();
    buffered.read_line(&mut command).await?;
    match command.trim() {
        "BRIDGE" => {
            // Keep the buffered reader: the bridge command and the peer's hello can
            // arrive in one socket read, and `into_inner` would discard those bytes.
            let (mut reader, mut writer) = tokio::io::split(buffered);
            daemon
                .serve_peer(&mut reader, &mut writer, "incoming SSH", shutdown, None)
                .await
        }
        "MONITOR" => serve_monitor(daemon, buffered.into_inner(), shutdown).await,
        "STATUS" => {
            let encoded = serde_json::to_vec(&daemon.status().await)?;
            let mut stream = buffered.into_inner();
            stream.write_all(&encoded).await?;
            stream.write_all(b"\n").await?;
            Ok(())
        }
        _ => bail!("unknown local socket command"),
    }
}

async fn serve_monitor(
    daemon: Arc<Daemon>,
    mut stream: UnixStream,
    mut shutdown: watch::Receiver<bool>,
) -> Result<()> {
    let mut events = daemon.events.subscribe();
    loop {
        tokio::select! {
            event = events.recv() => match event {
                Ok(event) => {
                    stream.write_all(&serde_json::to_vec(&event)?).await?;
                    stream.write_all(b"\n").await?;
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(broadcast::error::RecvError::Closed) => return Ok(()),
            },
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    return Ok(());
                }
            }
        }
    }
}

async fn dial_loop(daemon: Arc<Daemon>, peer: PeerConfig, mut shutdown: watch::Receiver<bool>) {
    let mut backoff = Duration::from_secs(1);
    loop {
        let established = AtomicBool::new(false);
        let result = async {
            let mut child = ssh::start_bridge(&peer.ssh_command)?;
            let mut writer = child.stdin.take().context("SSH bridge stdin unavailable")?;
            let mut reader = child.stdout.take().context("SSH bridge stdout unavailable")?;
            let result = Arc::clone(&daemon)
                .serve_peer(
                    &mut reader,
                    &mut writer,
                    &peer.name,
                    shutdown.clone(),
                    Some(&established),
                )
                .await;
            let _ = child.kill().await;
            result
        }
        .await;
        if established.load(Ordering::Acquire) {
            backoff = Duration::from_secs(1);
        }
        if let Err(error) = result {
            warn!(peer = %peer.name, %error, "peer connection failed");
        }
        tokio::select! {
            () = tokio::time::sleep(backoff) => {
                backoff = (backoff * 2).min(Duration::from_secs(10));
            }
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    return;
                }
            }
        }
    }
}

pub async fn bridge() -> Result<()> {
    let socket = paths()?.socket;
    let mut stream = UnixStream::connect(&socket)
        .await
        .with_context(|| format!("connect to daemon at {}", socket.display()))?;
    stream.write_all(b"BRIDGE\n").await?;
    let (mut socket_reader, mut socket_writer) = stream.into_split();
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    tokio::select! {
        result = tokio::io::copy(&mut stdin, &mut socket_writer) => { result?; }
        result = tokio::io::copy(&mut socket_reader, &mut stdout) => { result?; }
    }
    Ok(())
}

pub async fn query_status() -> Result<Status> {
    let socket = paths()?.socket;
    let mut stream = UnixStream::connect(&socket).await?;
    stream.write_all(b"STATUS\n").await?;
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line).await?;
    Ok(serde_json::from_str(&line)?)
}

pub async fn connect_monitor() -> Result<BufReader<UnixStream>> {
    let socket = paths()?.socket;
    let mut stream = UnixStream::connect(&socket).await?;
    stream.write_all(b"MONITOR\n").await?;
    Ok(BufReader::new(stream))
}

fn remove_stale_socket(path: &Path) -> Result<()> {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return Ok(());
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        if !metadata.file_type().is_socket() {
            bail!("refusing to replace non-socket path {}", path.display());
        }
    }
    std::fs::remove_file(path)?;
    Ok(())
}

#[cfg(unix)]
fn set_socket_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_socket_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = terminate.recv() => {}
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(test)]
mod tests {
    use tokio::io::{duplex, split};
    use tokio::time::timeout;

    use super::*;
    use crate::clipboard::test_support::MockClipboard;
    use crate::model::Representation;

    #[test]
    fn status_from_an_older_daemon_defaults_version_fields() {
        let status: Status = serde_json::from_value(serde_json::json!({
            "running": true,
            "node_id": Uuid::new_v4(),
            "node_name": "older-mac",
            "clipboard_backend": "NSPasteboard",
            "connected_peers": []
        }))
        .unwrap();

        assert_eq!(status.version, "legacy");
        assert_eq!(status.desired_version, "legacy");
        assert!(status.configured_peers.is_empty());
        assert!(status.peers.is_empty());
    }

    #[tokio::test]
    async fn bridge_command_preserves_a_coalesced_peer_hello() {
        let config = Config::default();
        let max_bytes = config.max_bytes;
        let clipboard = Arc::new(MockClipboard::default());
        let daemon = Daemon::new(config, clipboard);
        let (mut client, server) = UnixStream::pair().unwrap();
        client.write_all(b"BRIDGE\n").await.unwrap();
        write_message(
            &mut client,
            &Message::Hello {
                node_id: Uuid::new_v4(),
                node_name: "remote-mac".into(),
                app_version: Some(CURRENT_VERSION.into()),
                desired_version: Some(CURRENT_VERSION.into()),
            },
            max_bytes,
        )
        .await
        .unwrap();

        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let task = tokio::spawn(handle_local(Arc::clone(&daemon), server, shutdown_rx));
        assert!(matches!(
            read_message(&mut client, max_bytes).await.unwrap(),
            Message::Hello { .. }
        ));
        timeout(Duration::from_secs(1), async {
            loop {
                if daemon.status().await.connected_peers == ["remote-mac"] {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        shutdown_tx.send(true).unwrap();
        task.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn update_versions_are_announced_and_peer_hints_are_forwarded() {
        let config = Config::default();
        let max_bytes = config.max_bytes;
        let clipboard = Arc::new(MockClipboard::default());
        let (desired_tx, _) = watch::channel(CURRENT_VERSION.to_owned());
        let (hint_tx, mut hint_rx) = mpsc::unbounded_channel();
        let daemon = Daemon::with_updates(config, clipboard, desired_tx.clone(), hint_tx);
        let (mut peer, server) = duplex(4096);
        let (mut reader, mut writer) = split(server);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let task = tokio::spawn(async move {
            daemon
                .serve_peer(&mut reader, &mut writer, "test", shutdown_rx, None)
                .await
        });
        assert!(matches!(
            read_message(&mut peer, max_bytes).await.unwrap(),
            Message::Hello { .. }
        ));
        write_message(
            &mut peer,
            &Message::Hello {
                node_id: Uuid::new_v4(),
                node_name: "peer".into(),
                app_version: Some(CURRENT_VERSION.into()),
                desired_version: Some(CURRENT_VERSION.into()),
            },
            max_bytes,
        )
        .await
        .unwrap();

        timeout(Duration::from_secs(1), async {
            while desired_tx.receiver_count() == 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        desired_tx.send("9.0.0".into()).unwrap();
        assert!(matches!(
            read_message(&mut peer, max_bytes).await.unwrap(),
            Message::UpdateAvailable { version, .. } if version == "9.0.0"
        ));
        write_message(
            &mut peer,
            &Message::UpdateAvailable {
                update_id: Uuid::new_v4(),
                version: "9.1.0".into(),
            },
            max_bytes,
        )
        .await
        .unwrap();
        assert_eq!(
            timeout(Duration::from_secs(1), hint_rx.recv()).await.unwrap(),
            Some("9.1.0".into())
        );

        shutdown_tx.send(true).unwrap();
        task.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn relays_a_clip_between_peers_and_applies_it_locally() {
        let config = Config::default();
        let clipboard = Arc::new(MockClipboard::default());
        let daemon = Daemon::new(config.clone(), clipboard.clone());
        let (_shutdown_tx, shutdown_rx) = watch::channel(false);
        let (peer_a, server_a) = duplex(16 * 1024);
        let (peer_b, server_b) = duplex(16 * 1024);
        let (mut server_a_read, mut server_a_write) = split(server_a);
        let (mut server_b_read, mut server_b_write) = split(server_b);
        let daemon_a = Arc::clone(&daemon);
        let shutdown_a = shutdown_rx.clone();
        tokio::spawn(async move {
            daemon_a
                .serve_peer(&mut server_a_read, &mut server_a_write, "a", shutdown_a, None)
                .await
        });
        let daemon_b = Arc::clone(&daemon);
        tokio::spawn(async move {
            daemon_b
                .serve_peer(&mut server_b_read, &mut server_b_write, "b", shutdown_rx, None)
                .await
        });
        let (mut a_read, mut a_write) = split(peer_a);
        let (mut b_read, mut b_write) = split(peer_b);
        for (reader, writer, name) in [
            (&mut a_read, &mut a_write, "peer-a"),
            (&mut b_read, &mut b_write, "peer-b"),
        ] {
            assert!(matches!(
                read_message(reader, config.max_bytes).await.unwrap(),
                Message::Hello { .. }
            ));
            write_message(
                writer,
                &Message::Hello {
                    node_id: Uuid::new_v4(),
                    node_name: name.into(),
                    app_version: Some(CURRENT_VERSION.into()),
                    desired_version: Some(CURRENT_VERSION.into()),
                },
                config.max_bytes,
            )
            .await
            .unwrap();
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
        let clip = Clip::new(
            Uuid::new_v4(),
            vec![Representation {
                item: 0,
                format: "image/heic".into(),
                data: vec![1, 2, 3, 4],
            }],
        );
        write_clip(&mut a_write, &clip, config.max_bytes).await.unwrap();
        let relayed = timeout(
            Duration::from_secs(1),
            read_message(&mut b_read, config.max_bytes),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(relayed, Message::Clip(clip.clone()));
        let applied = timeout(Duration::from_secs(1), async {
            loop {
                if let Some(snapshot) = clipboard.capture().await.unwrap()
                    && snapshot.representations == clip.representations
                {
                    break snapshot;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(applied.representations, clip.representations);
    }

    #[test]
    fn refuses_to_delete_a_non_socket_path() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("daemon.sock");
        std::fs::write(&path, b"important").unwrap();
        assert!(remove_stale_socket(&path).is_err());
        assert_eq!(std::fs::read(path).unwrap(), b"important");
    }
}
