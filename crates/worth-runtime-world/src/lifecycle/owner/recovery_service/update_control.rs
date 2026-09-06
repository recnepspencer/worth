use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::recovery::ProductUnpublishedRecoveryHandle;
const RECOVERY_UPDATE_PAUSE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub(super) struct RecoveryUpdatePause {
    reached: SyncSender<()>,
    release: Arc<(Mutex<bool>, Condvar)>,
    /// Set when the paused update gave up waiting and resumed on its own.
    timed_out: AtomicBool,
}

pub(super) struct RecoveryUpdatePauseGuard {
    handle: ProductUnpublishedRecoveryHandle,
    pause: Arc<RecoveryUpdatePause>,
}

static RECOVERY_UPDATE_PAUSES: OnceLock<
    Mutex<HashMap<ProductUnpublishedRecoveryHandle, Arc<RecoveryUpdatePause>>>,
> = OnceLock::new();

fn recovery_update_pause_slot(
) -> &'static Mutex<HashMap<ProductUnpublishedRecoveryHandle, Arc<RecoveryUpdatePause>>> {
    RECOVERY_UPDATE_PAUSES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn install_test_recovery_update_pause(
    handle: ProductUnpublishedRecoveryHandle,
    reached: SyncSender<()>,
) -> RecoveryUpdatePauseGuard {
    let pause = Arc::new(RecoveryUpdatePause {
        reached,
        release: Arc::new((Mutex::new(false), Condvar::new())),
        timed_out: AtomicBool::new(false),
    });
    let mut installed = recovery_update_pause_slot()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    assert!(
        !installed.contains_key(&handle),
        "only one recovery update pause may be installed for a handle"
    );
    installed.insert(handle.clone(), Arc::clone(&pause));
    RecoveryUpdatePauseGuard { handle, pause }
}

impl RecoveryUpdatePause {
    fn wait_for_release(&self) {
        if self.reached.send(()).is_err() {
            return;
        }
        let deadline = Instant::now() + RECOVERY_UPDATE_PAUSE_TIMEOUT;
        let (opened, signal) = &*self.release;
        let mut opened = opened.lock().unwrap_or_else(|error| error.into_inner());
        while !*opened {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                self.note_timeout();
                return;
            };
            let (next, result) = signal
                .wait_timeout(opened, remaining)
                .unwrap_or_else(|error| error.into_inner());
            opened = next;
            // A release that lands in the same instant the wait expires is a
            // release, not a timeout: the loop condition decides.
            if result.timed_out() && !*opened {
                self.note_timeout();
                return;
            }
        }
    }

    fn note_timeout(&self) {
        self.timed_out.store(true, Ordering::SeqCst);
    }

    fn timed_out(&self) -> bool {
        self.timed_out.load(Ordering::SeqCst)
    }

    fn release(&self) {
        let (opened, signal) = &*self.release;
        let mut opened = opened.lock().unwrap_or_else(|error| error.into_inner());
        *opened = true;
        signal.notify_all();
    }
}

impl Drop for RecoveryUpdatePauseGuard {
    fn drop(&mut self) {
        self.pause.release();
        let mut installed = recovery_update_pause_slot()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let owned_registration = installed
            .get(&self.handle)
            .is_some_and(|current| Arc::ptr_eq(current, &self.pause));
        if owned_registration {
            installed.remove(&self.handle);
        }
        drop(installed);
        // A pause that resumed on its own describes an update the test never
        // controlled, so the test fails here by name.
        assert!(
            !self.pause.timed_out() || std::thread::panicking(),
            "recovery update pause was never released within {RECOVERY_UPDATE_PAUSE_TIMEOUT:?}"
        );
    }
}

pub(super) fn pause_test_recovery_update(handle: &ProductUnpublishedRecoveryHandle) {
    let pause = recovery_update_pause_slot()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(handle)
        .cloned();
    if let Some(pause) = pause {
        pause.wait_for_release();
    }
}
