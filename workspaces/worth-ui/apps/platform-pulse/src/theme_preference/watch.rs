use super::{PlatformPulseThemePreference, PlatformPulseThemePreferenceDenial as Denial};
use notify::Watcher;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal;

const FILE_NAME: &str = "platform-pulse-theme.json";
const MAXIMUM_BYTES: u64 = 4096;
const QUIET_INTERVAL: Duration = Duration::from_millis(50);
const MAXIMUM_SETTLE: Duration = Duration::from_millis(250);

#[derive(Default)]
struct PendingPreference {
    observation: Option<Result<Option<PlatformPulseThemePreference>, Denial>>,
    signal: Option<PlatformPulseApplicationReadinessSignal>,
}

pub(crate) struct PlatformPulseThemePreferenceWatch {
    watcher: Option<notify::RecommendedWatcher>,
    wake: mpsc::SyncSender<()>,
    stop: Arc<AtomicBool>,
    pending: Arc<Mutex<PendingPreference>>,
    worker: Option<std::thread::JoinHandle<()>>,
    admitted: Option<PlatformPulseThemePreference>,
}

impl PlatformPulseThemePreferenceWatch {
    pub(crate) fn open(root: &Path) -> Result<Self, Denial> {
        let root = root.canonicalize().map_err(|_| Denial::WatchUnavailable)?;
        let (wake, notifications) = mpsc::sync_channel(1);
        let callback_wake = wake.clone();
        let failed = Arc::new(AtomicBool::new(false));
        let callback_failed = Arc::clone(&failed);
        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                let relevant = event.as_ref().map_or(true, |event| {
                    !matches!(event.kind, notify::EventKind::Access(_))
                        && event
                            .paths
                            .iter()
                            .any(|path| path.file_name().is_some_and(|name| name == FILE_NAME))
                });
                if !relevant {
                    return;
                }
                if event.is_err() {
                    callback_failed.store(true, Ordering::Release);
                }
                // A single wake represents desired-file dirtiness, never a command.
                let _ = callback_wake.try_send(());
            })
            .map_err(|_| Denial::WatchUnavailable)?;
        watcher
            .watch(&root, notify::RecursiveMode::NonRecursive)
            .map_err(|_| Denial::WatchUnavailable)?;
        let pending = Arc::new(Mutex::new(PendingPreference::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_pending = Arc::clone(&pending);
        let worker_stop = Arc::clone(&stop);
        let worker = std::thread::Builder::new()
            .name("pulse-theme-preference".into())
            .spawn(move || {
                run(
                    root.join(FILE_NAME),
                    notifications,
                    failed,
                    worker_stop,
                    worker_pending,
                )
            })
            .map_err(|_| Denial::WatchUnavailable)?;
        let _ = wake.try_send(());
        Ok(Self {
            watcher: Some(watcher),
            wake,
            stop,
            pending,
            worker: Some(worker),
            admitted: None,
        })
    }

    pub(crate) fn install_readiness(&self, signal: PlatformPulseApplicationReadinessSignal) {
        let mut pending = self
            .pending
            .lock()
            .expect("theme observation mutex remains usable");
        pending.signal = Some(signal);
        if pending.observation.is_some() {
            pending.signal.as_ref().unwrap().signal();
        }
    }

    pub(crate) fn take_changed(&mut self) -> Result<Option<PlatformPulseThemePreference>, Denial> {
        let observation = self
            .pending
            .lock()
            .expect("theme observation mutex remains usable")
            .observation
            .take();
        let Some(Some(next)) = observation.transpose()? else {
            return Ok(None);
        };
        if next.schema_version != 1 {
            return Err(Denial::UnsupportedVersion);
        }
        if self.admitted == Some(next) {
            return Ok(None);
        }
        if next.revision == 0
            || self
                .admitted
                .is_some_and(|prior| next.revision <= prior.revision)
        {
            return Err(Denial::StaleRevision);
        }
        self.admitted = Some(next);
        Ok(Some(next))
    }

    pub(crate) fn shutdown(mut self) -> Result<(), Denial> {
        self.close()
    }

    fn close(&mut self) -> Result<(), Denial> {
        self.stop.store(true, Ordering::Release);
        let _ = self.wake.try_send(());
        self.watcher.take();
        let joined = self.worker.take().map_or(Ok(()), |worker| {
            worker.join().map_err(|_| Denial::WorkerPanicked)
        });
        let mut pending = self
            .pending
            .lock()
            .expect("theme observation mutex remains usable");
        pending.observation = None;
        pending.signal = None;
        joined
    }
}

impl Drop for PlatformPulseThemePreferenceWatch {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

fn run(
    path: PathBuf,
    notifications: mpsc::Receiver<()>,
    failed: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    pending: Arc<Mutex<PendingPreference>>,
) {
    while notifications.recv().is_ok() {
        if stop.load(Ordering::Acquire) {
            return;
        }
        let deadline = Instant::now() + MAXIMUM_SETTLE;
        while Instant::now() < deadline {
            match notifications.recv_timeout(
                QUIET_INTERVAL.min(deadline.saturating_duration_since(Instant::now())),
            ) {
                Ok(()) if !stop.load(Ordering::Acquire) => {}
                Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => return,
                Err(mpsc::RecvTimeoutError::Timeout) => break,
            }
        }
        if stop.load(Ordering::Acquire) {
            return;
        }
        let observation = if failed.load(Ordering::Acquire) {
            Err(Denial::WatchUnavailable)
        } else {
            read(&path)
        };
        let terminal = matches!(
            observation,
            Err(Denial::WatchUnavailable | Denial::ReadUnavailable)
        );
        let mut pending = pending
            .lock()
            .expect("theme observation mutex remains usable");
        pending.observation = Some(observation);
        if let Some(signal) = &pending.signal {
            signal.signal();
        }
        if terminal {
            return;
        }
    }
}

fn read(path: &Path) -> Result<Option<PlatformPulseThemePreference>, Denial> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(Denial::ReadUnavailable),
    };
    let mut bytes = Vec::new();
    file.take(MAXIMUM_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Denial::ReadUnavailable)?;
    if bytes.len() as u64 > MAXIMUM_BYTES {
        return Err(Denial::Oversized);
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|_| Denial::InvalidRecord)
}
