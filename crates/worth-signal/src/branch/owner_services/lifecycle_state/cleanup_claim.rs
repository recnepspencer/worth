use std::sync::atomic::Ordering;

use super::{SignalOwnerLifecycleState, OWNER_CLOSED, OWNER_CLOSING};

pub(in crate::branch::owner_services) struct SignalOwnerCleanupClaim<'a> {
    pub(super) lifecycle: &'a SignalOwnerLifecycleState,
    pub(super) completed: bool,
}

impl SignalOwnerCleanupClaim<'_> {
    pub(in crate::branch::owner_services) fn complete(mut self) {
        let _gate = self.lifecycle.lock_transition_gate();
        self.lifecycle
            .phase_and_count
            .compare_exchange(
                OWNER_CLOSING,
                OWNER_CLOSED,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .expect("Signal owner cleanup completes only after every admission releases");
        self.lifecycle
            .cleanup_claimed
            .store(false, Ordering::Release);
        self.completed = true;
        self.lifecycle.drain.notify_all();
    }
}

impl Drop for SignalOwnerCleanupClaim<'_> {
    fn drop(&mut self) {
        if self.completed {
            return;
        }
        let _gate = self.lifecycle.lock_transition_gate();
        self.lifecycle
            .cleanup_claimed
            .store(false, Ordering::Release);
        self.lifecycle.drain.notify_all();
    }
}
