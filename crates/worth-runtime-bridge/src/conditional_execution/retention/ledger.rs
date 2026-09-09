use crate::policy::BridgeConditionalRetentionBudget;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::conditional_execution) enum BridgeRetentionDenial {
    InvalidBudget,
    Closed,
    ClocksExhausted,
    IntentsExhausted,
    DefinitionCandidatesExhausted,
    BytesExhausted,
    PreparationExhausted,
    Quarantined,
}

/// This lock covers checked accounting only, never providers or component work.
#[derive(Debug)]
pub(in crate::conditional_execution) struct BridgeRetentionLedger {
    budget: BridgeConditionalRetentionBudget,
    state: Mutex<Usage>,
}

#[derive(Debug, Default)]
struct Usage {
    closed: bool,
    clocks: usize,
    intents: usize,
    definition_candidates: usize,
    bytes: u64,
}

/// Retains accounting without retaining the operational Bridge root.
#[derive(Debug)]
pub(in crate::conditional_execution) struct BridgeRetentionReservation {
    ledger: Arc<BridgeRetentionLedger>,
    clocks: usize,
    intents: usize,
    definition_candidates: usize,
    bytes: u64,
}

impl BridgeRetentionLedger {
    pub(in crate::conditional_execution) fn new(
        budget: BridgeConditionalRetentionBudget,
    ) -> Result<Arc<Self>, BridgeRetentionDenial> {
        if !budget.is_valid() {
            return Err(BridgeRetentionDenial::InvalidBudget);
        }
        Ok(Arc::new(Self {
            budget,
            state: Mutex::new(Usage::default()),
        }))
    }

    pub(in crate::conditional_execution) fn maximum_preparation_visits(&self) -> usize {
        self.budget.maximum_preparation_visits
    }

    /// Reserves total retained storage. The semantic owner includes its embedded
    /// reservation in its own layout; the ledger adds no second handle charge.
    pub(in crate::conditional_execution) fn reserve(
        self: &Arc<Self>,
        clocks: usize,
        intents: usize,
        bytes: u64,
    ) -> Result<BridgeRetentionReservation, BridgeRetentionDenial> {
        use BridgeRetentionDenial as D;
        let mut state = self.state.lock().map_err(|_| D::Quarantined)?;
        if state.closed {
            return Err(D::Closed);
        }
        let clocks_total = state
            .clocks
            .checked_add(clocks)
            .filter(|n| *n <= self.budget.maximum_managed_clocks)
            .ok_or(D::ClocksExhausted)?;
        let intents_total = state
            .intents
            .checked_add(intents)
            .filter(|n| *n <= self.budget.maximum_reserved_temporal_intents)
            .ok_or(D::IntentsExhausted)?;
        let bytes_total = state
            .bytes
            .checked_add(bytes)
            .filter(|n| *n <= self.budget.maximum_retained_bytes)
            .ok_or(D::BytesExhausted)?;
        state.clocks = clocks_total;
        state.intents = intents_total;
        state.bytes = bytes_total;
        Ok(BridgeRetentionReservation {
            ledger: Arc::clone(self),
            clocks,
            intents,
            definition_candidates: 0,
            bytes,
        })
    }

    pub(in crate::conditional_execution) fn reserve_definition_candidate(
        self: &Arc<Self>,
        bytes: u64,
    ) -> Result<BridgeRetentionReservation, BridgeRetentionDenial> {
        use BridgeRetentionDenial as D;
        let mut state = self.state.lock().map_err(|_| D::Quarantined)?;
        if state.closed {
            return Err(D::Closed);
        }
        let definitions_total = state
            .definition_candidates
            .checked_add(1)
            .filter(|n| *n <= self.budget.maximum_retained_definition_candidates)
            .ok_or(D::DefinitionCandidatesExhausted)?;
        let bytes_total = state
            .bytes
            .checked_add(bytes)
            .filter(|n| *n <= self.budget.maximum_retained_bytes)
            .ok_or(D::BytesExhausted)?;
        state.definition_candidates = definitions_total;
        state.bytes = bytes_total;
        Ok(BridgeRetentionReservation {
            ledger: Arc::clone(self),
            clocks: 0,
            intents: 0,
            definition_candidates: 1,
            bytes,
        })
    }

    pub(in crate::conditional_execution) fn close(&self) {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .closed = true;
    }

    pub(in crate::conditional_execution) fn observation(
        &self,
    ) -> super::BridgeConditionalRetentionObservation {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::BridgeConditionalRetentionObservation::new(
            state.clocks,
            state.intents,
            state.definition_candidates,
            state.bytes,
        )
    }

    #[cfg(test)]
    pub(in crate::conditional_execution) fn usage(&self) -> (usize, usize, u64) {
        let state = self.state.lock().unwrap();
        (state.clocks, state.intents, state.bytes)
    }
}

impl Drop for BridgeRetentionReservation {
    fn drop(&mut self) {
        let mut state = self
            .ledger
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.clocks = state
            .clocks
            .checked_sub(self.clocks)
            .expect("reservation owns clocks");
        state.intents = state
            .intents
            .checked_sub(self.intents)
            .expect("reservation owns intents");
        state.definition_candidates = state
            .definition_candidates
            .checked_sub(self.definition_candidates)
            .expect("reservation owns definition candidates");
        state.bytes = state
            .bytes
            .checked_sub(self.bytes)
            .expect("reservation owns bytes");
    }
}
