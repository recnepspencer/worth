//! Accepted Signal-owned charges, excluding work done by a compute callback.
use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::rc::Rc;
thread_local! { static LISTENERS: Cell<usize> = const { Cell::new(0) }; }
thread_local! { static CHARGES: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) }; }
thread_local! {
    static REFUSALS: RefCell<Vec<crate::data::error::SignalCheckpointDenial>> = const { RefCell::new(Vec::new()) };
}

/// Owns observation on this thread without granting execution authority.
#[must_use = "the observation ends when its guard is dropped"]
pub struct SignalRequestWorkObservation {
    thread_bound: PhantomData<Rc<()>>,
}

impl SignalRequestWorkObservation {
    /// Collect checkpoint work until the last listener on this thread drops.
    pub fn install_on_this_thread() -> Self {
        LISTENERS.with(|listeners| {
            listeners.set(
                listeners
                    .get()
                    .checked_add(1)
                    .expect("listener count overflow"),
            );
        });
        Self {
            thread_bound: PhantomData,
        }
    }
}

impl Drop for SignalRequestWorkObservation {
    fn drop(&mut self) {
        LISTENERS.with(|listeners| {
            let remaining = listeners.get() - 1;
            listeners.set(remaining);
            if remaining == 0 {
                CHARGES.with_borrow_mut(|charges| *charges = Vec::new());
                REFUSALS.with_borrow_mut(|refusals| *refusals = Vec::new());
            }
        });
    }
}

pub(super) fn record(units: usize, result: &Result<(), super::RetainedStoragePreparationDenial>) {
    if LISTENERS.with(Cell::get) == 0 {
        return;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_records_only_while_a_thread_bound_listener_is_owned() {
        record(11, &Ok(()));
        assert!(signal_request_charges_on_this_thread_for_test().is_empty());
        let outer = SignalRequestWorkObservation::install_on_this_thread();
        record(13, &Ok(()));
        {
            let _inner = SignalRequestWorkObservation::install_on_this_thread();
            record(15, &Ok(()));
        }
        assert_eq!(observed_signal_request_work_on_this_thread_for_test(), 28);
        assert_eq!(signal_request_charges_on_this_thread_for_test(), [13, 15]);
        record(17, &Ok(()));
        drop(outer);
        record(19, &Ok(()));
        assert!(signal_request_charges_on_this_thread_for_test().is_empty());
    }
}
