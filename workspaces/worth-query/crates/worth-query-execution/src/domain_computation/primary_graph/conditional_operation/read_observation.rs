//! Read-only observation of the conditional intent reader's consult boundary.
use std::cell::Cell;
thread_local! { static DEPTH: Cell<usize> = const { Cell::new(0) }; }

pub(super) struct IntentReadObservation {
    armed: bool,
}
impl IntentReadObservation {
    pub(super) fn enter() -> Self {
        let armed = crate::domain_computation::execution_observation::is_installed();
        if armed {
            DEPTH.set(DEPTH.get() + 1);
        }
        Self { armed }
    }
}
impl Drop for IntentReadObservation {
    fn drop(&mut self) {
        if self.armed {
            DEPTH.set(DEPTH.get() - 1);
        }
    }
}

/// Requires the owning ExecutionObservation guard; observes no mutation authority.
pub fn conditional_intent_read_is_active_for_test() -> bool {
    crate::domain_computation::execution_observation::is_installed() && DEPTH.get() != 0
}
