// Scheduled resource-bound evidence.  The large cases stay ignored in the
// ordinary court but are selected by the Scale roster with `--ignored`.
pub(crate) use std::thread;
pub(crate) use {std::sync::mpsc, std::sync::Arc, std::sync::Condvar, std::sync::Mutex};

pub(crate) use {
    worth_signal::facade::branch::SignalBranchAdvanceDenial,
    worth_signal::facade::branch::SignalBranchRetentionAcquisitionDenial,
    worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome,
    worth_signal::facade::branch::SignalOwnerCancellationSource,
};

pub(crate) use {
    crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld,
    crate::signal_owner_services::ordinary_context::adversarial::world::PROGRESS_BOUND,
};

#[derive(Default)]
pub(crate) struct ReleaseGate {
    pub(crate) released: Mutex<bool>,
    pub(crate) changed: Condvar,
}

impl ReleaseGate {
    pub(crate) fn wait(&self) {
        let mut released = self
            .released
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while !*released {
            released = self
                .changed
                .wait(released)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    pub(crate) fn release(&self) {
        let mut released = self
            .released
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *released = true;
        self.changed.notify_all();
    }
}

pub(crate) struct ReleaseOnDrop(pub(crate) Arc<ReleaseGate>);

impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        self.0.release();
    }
}
