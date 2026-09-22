use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use notify::Watcher;

use crate::bears_on_watched_file;

use super::decode::read_record;
use super::{
    PlatformPulseIntentInputRecord, PlatformPulseIntentInputWatchDenial, CHANNEL_CAPACITY,
    INPUT_FILE,
};

const WORKER_SETTLE_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AdmittedRevisionRelation {
    Duplicate,
    Stale,
    Successor,
}

pub(super) fn classify_revision(
    admitted_revision: u64,
    observed_revision: u64,
) -> AdmittedRevisionRelation {
    match observed_revision.cmp(&admitted_revision) {
        std::cmp::Ordering::Equal => AdmittedRevisionRelation::Duplicate,
        std::cmp::Ordering::Less => AdmittedRevisionRelation::Stale,
        std::cmp::Ordering::Greater => AdmittedRevisionRelation::Successor,
    }
}

pub(super) fn run_watch(
    root: PathBuf,
    mut admitted_revision: u64,
    stop: Arc<AtomicBool>,
    sender: mpsc::SyncSender<PlatformPulseIntentInputRecord>,
    terminal: Arc<Mutex<Option<PlatformPulseIntentInputWatchDenial>>>,
    readiness: Arc<Mutex<Option<crate::PlatformPulseApplicationReadinessSignal>>>,
) {
    let target = root.join(INPUT_FILE);
    let (notification_sender, notification_receiver) = mpsc::sync_channel(CHANNEL_CAPACITY);
    let notification_overflow = Arc::new(AtomicBool::new(false));
    let callback_overflow = Arc::clone(&notification_overflow);
    let watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        if !bears_on_watched_file(&event, INPUT_FILE) {
            return;
        }
        if notification_sender.try_send(event).is_err() {
            callback_overflow.store(true, Ordering::Release);
        }
    });
    let mut watcher = match watcher {
        Ok(watcher) => watcher,
        Err(error) => {
            store_terminal(
                &terminal,
                PlatformPulseIntentInputWatchDenial::Watcher(error.to_string()),
            );
            return;
        }
    };
    if let Err(error) = watcher.watch(&root, notify::RecursiveMode::NonRecursive) {
        store_terminal(
            &terminal,
            PlatformPulseIntentInputWatchDenial::Watcher(error.to_string()),
        );
        return;
    }
    while !stop.load(Ordering::Acquire) {
        if notification_overflow.swap(false, Ordering::AcqRel) {
            store_capacity_stop(&terminal);
            return;
        }
        match notification_receiver.recv_timeout(WORKER_SETTLE_INTERVAL) {
            Ok(Ok(_)) => match read_record(&target) {
                Ok(record) => match classify_revision(admitted_revision, record.revision()) {
                    AdmittedRevisionRelation::Duplicate => continue,
                    AdmittedRevisionRelation::Stale => {
                        store_terminal(
                            &terminal,
                            PlatformPulseIntentInputWatchDenial::StaleRevision {
                                active: admitted_revision,
                                observed: record.revision(),
                            },
                        );
                        return;
                    }
                    AdmittedRevisionRelation::Successor => {
                        admitted_revision = record.revision();
                        if sender.try_send(record).is_err() {
                            store_capacity_stop(&terminal);
                            signal_readiness(&readiness);
                            return;
                        }
                        signal_readiness(&readiness);
                    }
                },
                Err(denial) => {
                    store_terminal(&terminal, denial);
                    signal_readiness(&readiness);
                    return;
                }
            },
            Ok(Err(error)) => {
                store_terminal(
                    &terminal,
                    PlatformPulseIntentInputWatchDenial::Watcher(error.to_string()),
                );
                signal_readiness(&readiness);
                return;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        }
    }
}

fn signal_readiness(readiness: &Mutex<Option<crate::PlatformPulseApplicationReadinessSignal>>) {
    if let Some(readiness) = readiness
        .lock()
        .expect("intent readiness signal is not poisoned")
        .as_ref()
    {
        readiness.signal();
    }
}

fn store_capacity_stop(terminal: &Mutex<Option<PlatformPulseIntentInputWatchDenial>>) {
    store_terminal(
        terminal,
        PlatformPulseIntentInputWatchDenial::ChannelCapacityExceeded {
            capacity: CHANNEL_CAPACITY,
        },
    );
}

fn store_terminal(
    terminal: &Mutex<Option<PlatformPulseIntentInputWatchDenial>>,
    denial: PlatformPulseIntentInputWatchDenial,
) {
    let mut slot = terminal
        .lock()
        .expect("intent watcher terminal state is not poisoned");
    if slot.is_none() {
        *slot = Some(denial);
    }
}
