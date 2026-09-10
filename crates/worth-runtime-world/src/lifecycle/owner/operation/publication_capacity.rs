use std::sync::{Arc, Mutex};

use crate::budget::RuntimeWorldBudgetLimit;

#[derive(Debug)]
struct PublicationAttemptCapacityState {
    maximum: usize,
    active: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeWorldPublicationCapacityLedger {
    state: Arc<Mutex<PublicationAttemptCapacityState>>,
}

#[must_use = "a publication attempt capacity reservation must be retained or dropped"]
pub(crate) struct ReservedPublicationAttemptCapacity {
    ledger: RuntimeWorldPublicationCapacityLedger,
    armed: bool,
}

impl std::fmt::Debug for ReservedPublicationAttemptCapacity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReservedPublicationAttemptCapacity")
            .field("armed", &self.armed)
            .finish_non_exhaustive()
    }
}

impl Drop for ReservedPublicationAttemptCapacity {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let mut state = self
            .ledger
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        state.active = state
            .active
            .checked_sub(1)
            .expect("a live publication reservation owns one active slot");
        self.armed = false;
    }
}

impl RuntimeWorldPublicationCapacityLedger {
    pub(in crate::lifecycle::owner) fn new(limit: RuntimeWorldBudgetLimit) -> Self {
        Self {
            state: Arc::new(Mutex::new(PublicationAttemptCapacityState {
                maximum: limit.get(),
                active: 0,
            })),
        }
    }

    pub(crate) fn reserve(&self) -> Result<ReservedPublicationAttemptCapacity, ()> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.active >= state.maximum {
            return Err(());
        }
        state.active += 1;
        Ok(ReservedPublicationAttemptCapacity {
            ledger: self.clone(),
            armed: true,
        })
    }

    pub(crate) fn active(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .active
    }
}
