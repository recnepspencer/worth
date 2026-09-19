use std::sync::{Arc, Mutex, PoisonError};

use crate::runtime::WorthQueryConditionalEvaluationCacheBudget;

#[derive(Debug)]
pub(super) struct WorthQueryConditionalEvaluationRetentionLedger {
    maximum_entries: usize,
    maximum_bytes: u64,
    state: Mutex<WorthQueryConditionalEvaluationRetentionState>,
}

#[derive(Debug, Default)]
struct WorthQueryConditionalEvaluationRetentionState {
    entries: usize,
    bytes: u64,
}

#[derive(Debug)]
pub(super) struct WorthQueryConditionalEvaluationRetentionReservation {
    ledger: Arc<WorthQueryConditionalEvaluationRetentionLedger>,
    entries: usize,
    bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorthQueryConditionalEvaluationRetentionCapacity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorthQueryConditionalEvaluationRetentionSnapshot {
    pub(super) entries: usize,
    pub(super) bytes: u64,
}

impl WorthQueryConditionalEvaluationRetentionLedger {
    pub(super) fn new(budget: WorthQueryConditionalEvaluationCacheBudget) -> Arc<Self> {
        Arc::new(Self {
            maximum_entries: budget.maximum_retained_entries(),
            maximum_bytes: budget.maximum_retained_bytes(),
            state: Mutex::new(Default::default()),
        })
    }

    pub(super) fn reserve(
        self: &Arc<Self>,
        entries: usize,
        bytes: u64,
    ) -> Result<
        WorthQueryConditionalEvaluationRetentionReservation,
        WorthQueryConditionalEvaluationRetentionCapacity,
    > {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let next_entries = state
            .entries
            .checked_add(entries)
            .filter(|total| *total <= self.maximum_entries)
            .ok_or(WorthQueryConditionalEvaluationRetentionCapacity)?;
        let next_bytes = state
            .bytes
            .checked_add(bytes)
            .filter(|total| *total <= self.maximum_bytes)
            .ok_or(WorthQueryConditionalEvaluationRetentionCapacity)?;
        state.entries = next_entries;
        state.bytes = next_bytes;
        Ok(WorthQueryConditionalEvaluationRetentionReservation {
            ledger: Arc::clone(self),
            entries,
            bytes,
        })
    }

    pub(super) fn observe(&self) -> WorthQueryConditionalEvaluationRetentionSnapshot {
        let state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        WorthQueryConditionalEvaluationRetentionSnapshot {
            entries: state.entries,
            bytes: state.bytes,
        }
    }
}

impl Drop for WorthQueryConditionalEvaluationRetentionReservation {
    fn drop(&mut self) {
        let mut state = self
            .ledger
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        state.entries = state
            .entries
            .checked_sub(self.entries)
            .expect("reservation owns its Query cache entries");
        state.bytes = state
            .bytes
            .checked_sub(self.bytes)
            .expect("reservation owns its Query cache bytes");
    }
}
