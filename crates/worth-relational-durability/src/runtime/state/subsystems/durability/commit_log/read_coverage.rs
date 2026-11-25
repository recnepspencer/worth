//! Already admitted canonical history read through its original durable owner.
use crate::durability::data::DurableCheckpoint;
use std::sync::Arc;
use worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use worth_relational_storage::internal_port::storage::substrate::owned_state::RuntimeOwnedState;

use super::super::state::DurabilityState;

#[derive(Clone, Copy)]
pub struct DurableCanonicalCoverage<'a> {
    state: &'a RuntimeOwnedState<DurabilityState>,
}

impl<'a> DurableCanonicalCoverage<'a> {
    pub fn new(state: &'a RuntimeOwnedState<DurabilityState>) -> Self {
        Self { state }
    }

    pub fn log(&self) -> Vec<Arc<PositionedCanonicalCommit>> {
        self.state
            .read()
            .log
            .iter()
            .filter_map(|entry| entry.2.positioned_if_resident())
            .map(Arc::new)
            .collect()
    }

    pub fn checkpoints(&self) -> Vec<Arc<DurableCheckpoint>> {
        self.state.read().checkpoints.clone()
    }
}
