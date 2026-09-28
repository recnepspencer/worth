use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};

use serde::{Deserialize, Serialize};

mod decode;
#[cfg(test)]
mod tests;
mod watch;

use decode::read_record;
use watch::{register_watch, run_watch};

pub(super) const INPUT_FILE: &str = "platform-pulse-intent.json";
pub(super) const INPUT_IDENTITY: &str = "worth-ui.platform-pulse.intent-source";
pub(super) const INPUT_SCHEMA_VERSION: u16 = 1;
pub(super) const INPUT_BYTE_LIMIT: usize = 16_384;
pub(super) const CHANNEL_CAPACITY: usize = 16;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformPulseIntentInputOperability {
    Ready,
    Disabled,
    Denied,
    ConfirmationRequired,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformPulseExecutorGatePosture {
    Held,
    Released,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformPulseIntentInputRecord {
    pub(super) revision: u64,
    pub(super) operability: PlatformPulseIntentInputOperability,
    pub(super) executor_gate: PlatformPulseExecutorGatePosture,
    pub(super) query_denial_requested: bool,
}

pub enum PlatformPulseIntentInputEvent {
    Record(PlatformPulseIntentInputRecord),
    Failed(PlatformPulseIntentInputWatchDenial),
}

pub struct PlatformPulseIntentInputInstallation {
    initial: PlatformPulseIntentInputRecord,
    watch: PlatformPulseIntentInputWatch,
}

pub struct PlatformPulseIntentInputWatch {
    receiver: mpsc::Receiver<PlatformPulseIntentInputRecord>,
    terminal: Arc<Mutex<Option<PlatformPulseIntentInputWatchDenial>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    readiness: Arc<Mutex<Option<crate::PlatformPulseApplicationReadinessSignal>>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlatformPulseIntentInputWatchShutdownReceipt {
    worker_joined: bool,
    pending_event_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlatformPulseIntentInputWatchDenial {
    RootMetadata,
    RootNotDirectory,
    MissingInput,
    InputTooLarge { observed: usize, maximum: usize },
    Read(String),
    Decode(String),
    UnsupportedProtocol,
    UnsupportedVersion { observed: u16 },
    InvalidRevision,
    StaleRevision { active: u64, observed: u64 },
    Watcher(String),
    ChannelCapacityExceeded { capacity: usize },
    WorkerPanicked,
}

impl PlatformPulseIntentInputInstallation {
    pub fn open(root: &Path) -> Result<Self, PlatformPulseIntentInputWatchDenial> {
        let metadata = std::fs::metadata(root)
            .map_err(|_| PlatformPulseIntentInputWatchDenial::RootMetadata)?;
        if !metadata.is_dir() {
            return Err(PlatformPulseIntentInputWatchDenial::RootNotDirectory);
        }
        let target = root.join(INPUT_FILE);
        if !target.is_file() {
            return Err(PlatformPulseIntentInputWatchDenial::MissingInput);
        }
        // The watch is registered before the initial read, so a successor
        // written after that read always reaches the worker.
        let registered = register_watch(root)?;
        let initial = read_record(&target)?;
        let (sender, receiver) = mpsc::sync_channel(CHANNEL_CAPACITY);
        let terminal = Arc::new(Mutex::new(None));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_terminal = Arc::clone(&terminal);
        let worker_stop = Arc::clone(&stop);
        let readiness = Arc::new(Mutex::new(None));
        let worker_readiness = Arc::clone(&readiness);
        let admitted_revision = initial.revision;
        #[cfg(test)]
        let start_delay = tests::WORKER_START_DELAY.get();
        let worker = thread::Builder::new()
            .name("worth-ui-platform-pulse-intent-source".to_owned())
            .spawn(move || {
                #[cfg(test)]
                thread::sleep(start_delay);
                run_watch(
                    target,
                    registered,
                    admitted_revision,
                    worker_stop,
                    sender,
                    worker_terminal,
                    worker_readiness,
                )
            })
            .map_err(|error| PlatformPulseIntentInputWatchDenial::Watcher(error.to_string()))?;
        Ok(Self {
            initial,
            watch: PlatformPulseIntentInputWatch {
                receiver,
                terminal,
                stop,
                worker: Some(worker),
                readiness,
            },
        })
    }

    pub fn into_parts(
        self,
    ) -> (
        PlatformPulseIntentInputRecord,
        PlatformPulseIntentInputWatch,
    ) {
        (self.initial, self.watch)
    }
}

impl PlatformPulseIntentInputRecord {
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub const fn operability(&self) -> PlatformPulseIntentInputOperability {
        self.operability
    }

    pub const fn executor_gate(&self) -> PlatformPulseExecutorGatePosture {
        self.executor_gate
    }

    pub const fn mutable(&self) -> bool {
        true
    }

    pub const fn ready(&self) -> bool {
        !matches!(
            self.operability,
            PlatformPulseIntentInputOperability::Disabled
        )
    }

    pub const fn policy_allowed(&self) -> bool {
        !matches!(
            self.operability,
            PlatformPulseIntentInputOperability::Denied
        )
    }

    pub const fn confirmation_required(&self) -> bool {
        matches!(
            self.operability,
            PlatformPulseIntentInputOperability::ConfirmationRequired
        )
    }

    pub const fn executor_held(&self) -> bool {
        matches!(self.executor_gate, PlatformPulseExecutorGatePosture::Held)
    }

    pub const fn query_denial_requested(&self) -> bool {
        self.query_denial_requested
    }
}

impl PlatformPulseIntentInputWatch {
    pub fn install_readiness(&self, readiness: crate::PlatformPulseApplicationReadinessSignal) {
        *self
            .readiness
            .lock()
            .expect("intent readiness installation is not poisoned") = Some(readiness);
    }

    pub fn try_next(&mut self) -> Option<PlatformPulseIntentInputEvent> {
        match self.receiver.try_recv() {
            Ok(record) => Some(PlatformPulseIntentInputEvent::Record(record)),
            Err(mpsc::TryRecvError::Disconnected | mpsc::TryRecvError::Empty) => self
                .terminal
                .lock()
                .expect("intent watcher terminal state is not poisoned")
                .take()
                .map(PlatformPulseIntentInputEvent::Failed),
        }
    }

    pub fn shutdown(
        mut self,
    ) -> Result<PlatformPulseIntentInputWatchShutdownReceipt, PlatformPulseIntentInputWatchDenial>
    {
        self.stop.store(true, Ordering::Release);
        let worker_joined = match self.worker.take().map(JoinHandle::join) {
            Some(Ok(())) | None => true,
            Some(Err(_)) => return Err(PlatformPulseIntentInputWatchDenial::WorkerPanicked),
        };
        if let Some(denial) = self
            .terminal
            .lock()
            .expect("intent watcher terminal state is not poisoned")
            .take()
        {
            return Err(denial);
        }
        Ok(PlatformPulseIntentInputWatchShutdownReceipt {
            worker_joined,
            pending_event_count: self.receiver.try_iter().count(),
        })
    }
}

impl PlatformPulseIntentInputWatchShutdownReceipt {
    pub const fn worker_joined(self) -> bool {
        self.worker_joined
    }

    pub const fn pending_event_count(self) -> usize {
        self.pending_event_count
    }
}

impl std::fmt::Display for PlatformPulseIntentInputWatchDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RootMetadata => formatter.write_str("intent source metadata unavailable"),
            Self::RootNotDirectory => formatter.write_str("intent source is not a directory"),
            Self::MissingInput => formatter.write_str("intent source file is missing"),
            Self::InputTooLarge { observed, maximum } => {
                write!(
                    formatter,
                    "intent input is {observed} bytes; limit is {maximum}"
                )
            }
            Self::Read(detail) => write!(formatter, "intent input read: {detail}"),
            Self::Decode(detail) => write!(formatter, "intent input decode: {detail}"),
            Self::UnsupportedProtocol => formatter.write_str("unsupported intent input protocol"),
            Self::UnsupportedVersion { observed } => {
                write!(formatter, "unsupported intent input version {observed}")
            }
            Self::InvalidRevision => formatter.write_str("intent input revision must be nonzero"),
            Self::StaleRevision { active, observed } => {
                write!(
                    formatter,
                    "intent input revision {observed} is not after {active}"
                )
            }
            Self::Watcher(detail) => write!(formatter, "intent source watcher: {detail}"),
            Self::ChannelCapacityExceeded { capacity } => {
                write!(
                    formatter,
                    "intent source channel exceeded capacity {capacity}"
                )
            }
            Self::WorkerPanicked => formatter.write_str("intent source worker panicked"),
        }
    }
}
