use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::data::graph::{BranchMutationRecord, BranchStructuralDelta};
use crate::data::handle::NodeId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BranchMutationLedger {
    pub(crate) pending: BTreeMap<NodeId, BranchMutationRecord>,
    pub(crate) baseline_snapshot_id: Option<crate::state::SignalSnapshotId>,
    pub(crate) boundary_established: bool,
}

impl BranchMutationLedger {
    pub fn with_baseline_snapshot(
        mut self,
        snapshot_id: Option<crate::state::SignalSnapshotId>,
    ) -> Self {
        self.baseline_snapshot_id = snapshot_id;
        self.boundary_established = true;
        self
    }

    pub(crate) fn absorb_records(
        &mut self,
        records: impl IntoIterator<Item = (NodeId, BranchMutationRecord)>,
    ) {
        for (node, record) in records {
            self.pending.entry(node).or_default().absorb(record);
        }
    }

    pub fn clear_all(&mut self, baseline_snapshot_id: Option<crate::state::SignalSnapshotId>) {
        self.pending.clear();
        self.baseline_snapshot_id = baseline_snapshot_id;
        self.boundary_established = true;
    }
}
