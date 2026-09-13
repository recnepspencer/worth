use std::sync::{Arc, Condvar, Mutex, Weak};
use std::thread::ThreadId;

use super::super::basis::AdmittedSignalBranchBasisInner;
use super::super::SignalBranchRetentionAcquisitionDenial;

#[derive(Debug)]
pub(super) enum RegistryEntry {
    Acquiring(AcquiringEntry),
    Ready {
        registration_id: u64,
        basis: Weak<AdmittedSignalBranchBasisInner>,
    },
}

#[derive(Debug)]
pub(super) struct AcquiringEntry {
    pub(super) reservation_id: u64,
    pub(super) initiating_thread: ThreadId,
    pub(super) completion: Arc<SingleFlightCompletion>,
}

#[derive(Debug)]
enum CompletionState {
    Pending,
    Finished(
        Result<super::super::AdmittedSignalBranchBasis, SignalBranchRetentionAcquisitionDenial>,
    ),
}

/// Shared completion for one exact-basis reservation. It owns no basis or
/// lease until the claimant publishes a successful owner result.
#[derive(Debug)]
pub(super) struct SingleFlightCompletion {
    state: Mutex<CompletionState>,
    wake: Condvar,
    joined_waiters: Mutex<usize>,
    joined_wake: Condvar,
}

impl SingleFlightCompletion {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(CompletionState::Pending),
            wake: Condvar::new(),
            joined_waiters: Mutex::new(0),
            joined_wake: Condvar::new(),
        }
    }

    pub(super) fn record_joiner(&self) {
        let mut joined = self
            .joined_waiters
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *joined += 1;
        self.joined_wake.notify_all();
    }

    #[cfg(test)]
    pub(super) fn wait_for_joiners(&self, expected: usize) {
        let mut joined = self
            .joined_waiters
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while *joined < expected {
            joined = self
                .joined_wake
                .wait(joined)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    pub(super) fn finish(
        &self,
        result: Result<
            super::super::AdmittedSignalBranchBasis,
            SignalBranchRetentionAcquisitionDenial,
        >,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if matches!(*state, CompletionState::Pending) {
            *state = CompletionState::Finished(result);
            self.wake.notify_all();
        }
    }

    pub(super) fn wait(
        &self,
    ) -> Result<super::super::AdmittedSignalBranchBasis, SignalBranchRetentionAcquisitionDenial>
    {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while matches!(*state, CompletionState::Pending) {
            state = self
                .wake
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        match &*state {
            CompletionState::Pending => unreachable!("a completed Signal wait cannot be pending"),
            CompletionState::Finished(result) => result.clone(),
        }
    }
}
