//! One retained computation and the ledger ticket that lives exactly as long.
use super::retained_capacity::RetainedLineageCapacity;
use crate::domain_computation::primary_graph::application_contribution::RetainedComputation;
use std::ops::Deref;
use std::sync::Arc;

pub(in crate::domain_computation::primary_graph) struct CustodiedComputation {
    state: RetainedComputation,
    capacity: RetainedLineageCapacity,
}

impl CustodiedComputation {
    pub(super) fn new(state: RetainedComputation, capacity: RetainedLineageCapacity) -> Arc<Self> {
        assert_eq!(
            Self::retained_bytes_for(&state),
            Some(capacity.bytes()),
            "a completed state holds its full declared charge"
        );
        Arc::new(Self { state, capacity })
    }

    pub(super) fn retained_bytes_for(state: &RetainedComputation) -> Option<u64> {
        state
            .retained_bytes()?
            .checked_add(super::prepared_slot::arc_bytes::<Self>()?)
    }

    pub(super) fn admit_successor_growth(
        &mut self,
        bytes: u64,
    ) -> Result<(), super::super::WorthQueryOutputDemandDenial> {
        if bytes > self.capacity.bytes() {
            self.capacity
                .reserve_additional(bytes - self.capacity.bytes())?;
        }
        Ok(())
    }

    /// Called only after consuming every state holder. Shared handles cannot
    /// yield a reservation independently from the retained state.
    pub(super) fn into_capacity(self) -> RetainedLineageCapacity {
        let Self { state, capacity } = self;
        drop(state);
        capacity
    }
}

impl Deref for CustodiedComputation {
    type Target = RetainedComputation;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

#[cfg(test)]
pub(in crate::domain_computation::primary_graph) fn custodied_state_for_test(
    state: RetainedComputation,
) -> Arc<CustodiedComputation> {
    let bytes = CustodiedComputation::retained_bytes_for(&state).expect("a test prior is measured");
    let capacity = super::retained_capacity::LineageRetentionLedger::default()
        .reserve(bytes)
        .expect("a test prior uses real ledger custody");
    CustodiedComputation::new(state, capacity)
}
