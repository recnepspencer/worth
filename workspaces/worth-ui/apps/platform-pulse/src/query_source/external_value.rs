use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use notify::Watcher;
use serde::Deserialize;
use worth_ui::facade::query_binding::WorthUiScalarProjectionSourceRecord;
use worth_ui_platform_pulse::bears_on_watched_file;

const VALUE_FILE: &str = "platform-pulse-value.json";
const WORKER_SETTLE_INTERVAL: Duration = Duration::from_millis(100);
const READ_SETTLEMENT_INTERVAL: Duration = Duration::from_millis(5);
const MAXIMUM_READ_SETTLEMENT_ATTEMPTS: usize = 8;

pub(crate) enum PlatformPulseExternalValueEvent {
    Record(WorthUiScalarProjectionSourceRecord),
    Failed(PlatformPulseExternalValueWatchDenial),
}

pub(crate) struct PlatformPulseExternalValueWatch {
    receiver: mpsc::Receiver<PlatformPulseExternalValueEvent>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<Result<(), PlatformPulseExternalValueWatchDenial>>>,
    readiness: Arc<Mutex<Option<worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal>>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlatformPulseExternalValueWatchShutdownReceipt {
    worker_joined: bool,
    pending_event_count: usize,
}

#[derive(Debug)]
pub(crate) enum PlatformPulseExternalValueWatchDenial {
    RootMetadata,
    RootNotDirectory,
    Watcher(String),
    Read(String),
    Decode(String),
    Record(String),
    WorkerPanicked,
}

impl std::fmt::Display for PlatformPulseExternalValueWatchDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RootMetadata => formatter.write_str("source-root metadata unavailable"),
            Self::RootNotDirectory => formatter.write_str("source root is not a directory"),
            Self::Watcher(detail) => write!(formatter, "native watcher: {detail}"),
            Self::Read(detail) => write!(formatter, "value read: {detail}"),
            Self::Decode(detail) => write!(formatter, "value decode: {detail}"),
            Self::Record(detail) => write!(formatter, "native record: {detail}"),
            Self::WorkerPanicked => formatter.write_str("source worker panicked"),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlatformPulseExternalValue {
    status: String,
    revision: u64,
}

impl PlatformPulseExternalValueWatch {
    pub(crate) fn spawn(root: &Path) -> Result<Self, PlatformPulseExternalValueWatchDenial> {
        let metadata = std::fs::metadata(root)
            .map_err(|_| PlatformPulseExternalValueWatchDenial::RootMetadata)?;
        if !metadata.is_dir() {
            return Err(PlatformPulseExternalValueWatchDenial::RootNotDirectory);
        }
        let root = root.to_owned();
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let readiness = Arc::new(Mutex::new(None));
        let worker_readiness = Arc::clone(&readiness);
        let (sender, receiver) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("worth-ui-platform-pulse-query-source".to_owned())
            .spawn(move || run_watch(root, worker_stop, sender, worker_readiness))
            .map_err(|error| PlatformPulseExternalValueWatchDenial::Watcher(error.to_string()))?;
        Ok(Self {
            receiver,
            stop,
            worker: Some(worker),
            readiness,
        })
    }

    pub(crate) fn install_readiness(
        &self,
        readiness: worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal,
    ) {
        *self
            .readiness
            .lock()
            .expect("Query readiness installation is not poisoned") = Some(readiness);
    }

    pub(crate) fn try_next(&self) -> Option<PlatformPulseExternalValueEvent> {
        self.receiver.try_recv().ok()
    }

    pub(crate) fn shutdown(
        mut self,
    ) -> Result<PlatformPulseExternalValueWatchShutdownReceipt, PlatformPulseExternalValueWatchDenial>
    {
        self.stop.store(true, Ordering::Release);
        match self.worker.take().map(JoinHandle::join) {
            Some(Ok(result)) => {
                result?;
                Ok(PlatformPulseExternalValueWatchShutdownReceipt {
                    worker_joined: true,
                    pending_event_count: self.receiver.try_iter().count(),
                })
            }
            Some(Err(_)) => Err(PlatformPulseExternalValueWatchDenial::WorkerPanicked),
            None => Ok(PlatformPulseExternalValueWatchShutdownReceipt {
                worker_joined: true,
                pending_event_count: self.receiver.try_iter().count(),
            }),
        }
    }
}

impl PlatformPulseExternalValueWatchShutdownReceipt {
    pub(crate) fn worker_joined(self) -> bool {
        self.worker_joined
    }

    pub(crate) fn pending_event_count(self) -> usize {
        self.pending_event_count
    }
}

fn run_watch(
    root: PathBuf,
    stop: Arc<AtomicBool>,
    sender: mpsc::Sender<PlatformPulseExternalValueEvent>,
    readiness: Arc<Mutex<Option<worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal>>>,
) -> Result<(), PlatformPulseExternalValueWatchDenial> {
    let target = root.join(VALUE_FILE);
    let (notification_sender, notification_receiver) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        if bears_on_watched_file(&event, VALUE_FILE) {
            let _ = notification_sender.send(event);
        }
    })
    .map_err(|error| PlatformPulseExternalValueWatchDenial::Watcher(error.to_string()))?;
    watcher
        .watch(&root, notify::RecursiveMode::NonRecursive)
        .map_err(|error| PlatformPulseExternalValueWatchDenial::Watcher(error.to_string()))?;
    let mut admitted_revision = None;
    publish_exact_target(&target, &mut admitted_revision, &sender);
    signal_readiness(&readiness);
    while !stop.load(Ordering::Acquire) {
        match notification_receiver.recv_timeout(WORKER_SETTLE_INTERVAL) {
            Ok(Ok(_)) => {
                publish_exact_target(&target, &mut admitted_revision, &sender);
                signal_readiness(&readiness);
            }
            Ok(Err(error)) => {
                let denial = PlatformPulseExternalValueWatchDenial::Watcher(error.to_string());
                let _ = sender.send(PlatformPulseExternalValueEvent::Failed(denial));
                signal_readiness(&readiness);
                return Ok(());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
        }
    }
    Ok(())
}

fn signal_readiness(
    readiness: &Mutex<Option<worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal>>,
) {
    if let Some(readiness) = readiness
        .lock()
        .expect("Query readiness signal is not poisoned")
        .as_ref()
    {
        readiness.signal();
    }
}

fn publish_exact_target(
    target: &Path,
    admitted_revision: &mut Option<u64>,
    sender: &mpsc::Sender<PlatformPulseExternalValueEvent>,
) {
    let Some(bytes) = settle_exact_target_read(target, sender) else {
        return;
    };
    let value: PlatformPulseExternalValue = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            let _ = sender.send(PlatformPulseExternalValueEvent::Failed(
                PlatformPulseExternalValueWatchDenial::Decode(error.to_string()),
            ));
            return;
        }
    };
    if admitted_revision.is_some_and(|revision| revision >= value.revision) {
        return;
    }
    let revision = value.revision;
    match WorthUiScalarProjectionSourceRecord::new(value.status, revision) {
        Ok(record) => {
            *admitted_revision = Some(revision);
            let _ = sender.send(PlatformPulseExternalValueEvent::Record(record));
        }
        Err(error) => {
            let _ = sender.send(PlatformPulseExternalValueEvent::Failed(
                PlatformPulseExternalValueWatchDenial::Record(format!("{error:?}")),
            ));
        }
    }
}

fn settle_exact_target_read(
    target: &Path,
    sender: &mpsc::Sender<PlatformPulseExternalValueEvent>,
) -> Option<Vec<u8>> {
    for attempt in 0..MAXIMUM_READ_SETTLEMENT_ATTEMPTS {
        match std::fs::read(target) {
            Ok(bytes) => return Some(bytes),
            Err(error) if transient_read_error(&error) => {
                if attempt + 1 == MAXIMUM_READ_SETTLEMENT_ATTEMPTS {
                    if error.kind() != std::io::ErrorKind::NotFound {
                        let _ = sender.send(PlatformPulseExternalValueEvent::Failed(
                            PlatformPulseExternalValueWatchDenial::Read(error.to_string()),
                        ));
                    }
                    return None;
                }
                thread::sleep(READ_SETTLEMENT_INTERVAL);
            }
            Err(error) => {
                let _ = sender.send(PlatformPulseExternalValueEvent::Failed(
                    PlatformPulseExternalValueWatchDenial::Read(error.to_string()),
                ));
                return None;
            }
        }
    }
    unreachable!("the bounded read-settlement loop always returns")
}

fn transient_read_error(error: &std::io::Error) -> bool {
    if error.kind() == std::io::ErrorKind::NotFound {
        return true;
    }
    #[cfg(windows)]
    {
        matches!(error.raw_os_error(), Some(32 | 33))
    }
    #[cfg(not(windows))]
    {
        false
    }
}

#[cfg(all(test, windows))]
mod tests {
    use std::os::windows::fs::OpenOptionsExt;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    use super::{publish_exact_target, PlatformPulseExternalValueEvent};

    static NEXT_LOCKED_VALUE: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn windows_atomic_replacement_lock_settles_within_the_bounded_read_budget() {
        let ordinal = NEXT_LOCKED_VALUE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "worth-ui-platform-pulse-locked-query-{}-{ordinal}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("create locked-value fixture");
        let target = root.join("platform-pulse-value.json");
        std::fs::write(&target, br#"{"status":"ONLINE","revision":1}"#)
            .expect("write valid Query value");
        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        std::fs::OpenOptions::write(&mut options, true);
        let lock = options
            .share_mode(0)
            .open(&target)
            .expect("hold an exclusive Windows file lock");
        let release = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(15));
            drop(lock);
        });
        let (sender, receiver) = std::sync::mpsc::channel();
        let started = Instant::now();
        publish_exact_target(&target, &mut None, &sender);
        release.join().expect("release lock worker");
        assert!(started.elapsed() <= Duration::from_millis(100));
        assert!(matches!(
            receiver.recv_timeout(Duration::from_millis(20)),
            Ok(PlatformPulseExternalValueEvent::Record(_))
        ));
        std::fs::remove_dir_all(root).expect("remove locked-value fixture");
    }
}
