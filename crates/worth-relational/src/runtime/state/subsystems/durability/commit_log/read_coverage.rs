//! Already admitted canonical history read through its original durable owner.
use crate::durability::data::DurableCheckpoint;
use crate::history::data::PositionedCanonicalCommit;
use crate::storage::substrate::owned_state::RuntimeOwnedState;
use std::sync::Arc;

use super::super::state::DurabilityState;

#[derive(Clone, Copy)]
pub(crate) struct DurableCanonicalCoverage<'a> {
    state: &'a RuntimeOwnedState<DurabilityState>,
}

impl<'a> DurableCanonicalCoverage<'a> {
    pub(in crate::runtime::state::subsystems::durability) fn new(
        state: &'a RuntimeOwnedState<DurabilityState>,
    ) -> Self {
        Self { state }
    }

    pub(crate) fn log(&self) -> Vec<Arc<PositionedCanonicalCommit>> {
        self.state
            .read()
            .log
            .iter()
            .filter_map(|entry| entry.2.positioned_if_resident())
            .map(Arc::new)
            .collect()
    }

    pub(crate) fn checkpoints(&self) -> Vec<Arc<DurableCheckpoint>> {
        self.state.read().checkpoints.clone()
    }
}
