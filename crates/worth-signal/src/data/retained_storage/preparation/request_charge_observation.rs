//! Accepted Signal-owned charges, excluding work done by a compute callback.
use std::cell::RefCell;
thread_local! { static CHARGES: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) }; }
thread_local! {
    static REFUSALS: RefCell<Vec<crate::data::error::SignalCheckpointDenial>> = const { RefCell::new(Vec::new()) };
}
pub(super) fn record(units: usize, result: &Result<(), super::RetainedStoragePreparationDenial>) {
    use super::RetainedStoragePreparationDenial as Denial;
    match result {
        Ok(()) => CHARGES.with_borrow_mut(|charges| charges.push(units as u64)),
        Err(Denial::ExecutionStopped(cause)) => {
            REFUSALS.with_borrow_mut(|refusals| refusals.push(*cause))
        }
        Err(
            Denial::WorkExhausted { .. }
            | Denial::ChargeOverflow
            | Denial::ChargeUnderflow
            | Denial::RetainedExtentHistoryUnavailable,
        ) => {}
    }
}
/// Takes this thread's Signal checkpoint charges. This grants no execution authority.
pub fn signal_request_charges_on_this_thread_for_test() -> Vec<u64> {
    CHARGES.with_borrow_mut(std::mem::take)
}

/// Takes typed stops from this thread's request-owned Signal checkpoints.
pub fn signal_request_refusals_on_this_thread_for_test(
) -> Vec<crate::data::error::SignalCheckpointDenial> {
    REFUSALS.with_borrow_mut(std::mem::take)
}

/// Accepted checkpoint work since the current thread last took its charge records.
/// Reading this observation never opens a scope or changes the charge ledger.
pub fn observed_signal_request_work_on_this_thread_for_test() -> u64 {
    CHARGES.with_borrow(|charges| charges.iter().copied().sum())
}
