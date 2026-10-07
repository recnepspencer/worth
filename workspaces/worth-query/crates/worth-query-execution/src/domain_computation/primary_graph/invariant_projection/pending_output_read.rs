//! An actual projected read names one exact native output needing managed progress.
use super::super::output_lineage::{
    invalidation::RetainedConsumedOutputCapacity, RecordedSettlementIdentity,
};
use std::sync::Arc;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

/// Scheduling evidence only; it grants neither a current entity nor a commit.
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct RequestedOutputRead {
    identity: Arc<RecordedSettlementIdentity>,
    selected: PositionedRelationalSnapshot,
    _capacity: RetainedConsumedOutputCapacity,
}
impl RequestedOutputRead {
    pub(super) fn new(
        identity: Arc<RecordedSettlementIdentity>,
        selected: PositionedRelationalSnapshot,
        capacity: RetainedConsumedOutputCapacity,
    ) -> Self {
        Self {
            identity,
            selected,
            _capacity: capacity,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn identity(
        &self,
    ) -> &RecordedSettlementIdentity {
        &self.identity
    }
    pub(in crate::domain_computation::primary_graph) fn selected(
        &self,
    ) -> &PositionedRelationalSnapshot {
        &self.selected
    }
}
impl PartialEq for RequestedOutputRead {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity && self.selected == other.selected
    }
}
impl Eq for RequestedOutputRead {}
