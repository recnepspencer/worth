use crate::history::data::BranchId;

use crate::branch::reference::observation::RelationalBranchReferenceObservation;
use crate::branch::version::RelationalBranchVersion;

/// Read-only owner observation of one mutable branch-reference cell.
///
/// This is an evidence surface, not an authority constructor: callers can
/// compare every currentness axis, but they cannot turn the value back into a
/// transaction binding or mutate the cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalBranchReferenceState {
    runtime_instance_id: u64,
    branch_id: BranchId,
    observation: RelationalBranchReferenceObservation,
    truth_version: RelationalBranchVersion,
    lifecycle: crate::branch::lifecycle::RelationalBranchLifecyclePosture,
    fork_provenance: Option<RelationalBranchReferenceObservation>,
    fork_source_branch_id: Option<BranchId>,
}

impl RelationalBranchReferenceState {
    pub(crate) fn new(
        runtime_instance_id: u64,
        branch_id: BranchId,
        observation: RelationalBranchReferenceObservation,
        truth_version: RelationalBranchVersion,
        lifecycle: crate::branch::lifecycle::RelationalBranchLifecyclePosture,
        fork_provenance: Option<RelationalBranchReferenceObservation>,
        fork_source_branch_id: Option<BranchId>,
    ) -> Self {
        Self {
            runtime_instance_id,
            branch_id,
            observation,
            truth_version,
            lifecycle,
            fork_provenance,
            fork_source_branch_id,
        }
    }

    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub fn observation(&self) -> &RelationalBranchReferenceObservation {
        &self.observation
    }

    pub const fn truth_version(&self) -> RelationalBranchVersion {
        self.truth_version
    }

    pub const fn lifecycle_posture(
        &self,
    ) -> crate::branch::lifecycle::RelationalBranchLifecyclePosture {
        self.lifecycle
    }

    pub fn fork_provenance(&self) -> Option<&RelationalBranchReferenceObservation> {
        self.fork_provenance.as_ref()
    }

    pub fn fork_source_branch_id(&self) -> Option<&BranchId> {
        self.fork_source_branch_id.as_ref()
    }
}
