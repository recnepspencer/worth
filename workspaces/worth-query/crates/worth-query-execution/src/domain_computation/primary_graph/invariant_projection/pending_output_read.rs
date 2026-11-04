//! An actual projected read names one exact native output needing managed progress.
use super::super::output_lineage::RecordedSettlementIdentity;
use std::sync::Arc;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

/// Scheduling evidence only; it grants neither a current entity nor a commit.
#[derive(Clone, Debug)]
pub struct RequestedOutputRead {
    identity: Arc<RecordedSettlementIdentity>,
    selected: PositionedRelationalSnapshot,
}
impl RequestedOutputRead {
    pub(super) fn new(
        identity: Arc<RecordedSettlementIdentity>,
        selected: PositionedRelationalSnapshot,
    ) -> Self {
        Self { identity, selected }
    }
    pub fn identity(&self) -> &RecordedSettlementIdentity {
        &self.identity
    }
    pub fn selected(&self) -> &PositionedRelationalSnapshot {
        &self.selected
    }
}
impl PartialEq for RequestedOutputRead {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity && self.selected == other.selected
    }
}
impl Eq for RequestedOutputRead {}
