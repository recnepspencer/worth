use super::owner_binding::{RelationalOwnerServiceBinding, RelationalOwnerServiceLifecyclePosture};
use crate::branch::{
    ArchivedRelationalBranch, RelationalBranchArchiveDenial, RelationalBranchDeleteDenial,
    RelationalBranchDeletionOutcome, RelationalBranchIdentity, RelationalBranchTransitionStop,
};

/// Descriptive lifecycle posture of the runtime owner behind a service port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalOwnerLifecycleObservation {
    Open,
    Closing,
    Closed,
}

/// Cloneable branch-lifecycle service weakly bound to one Relational owner.
#[derive(Debug, Clone)]
pub struct RelationalBranchLifecyclePort {
    owner: RelationalOwnerServiceBinding,
}

impl RelationalBranchLifecyclePort {
    pub(super) fn new(owner: RelationalOwnerServiceBinding) -> Self {
        Self { owner }
    }

    pub fn archive_branch(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &RelationalBranchIdentity,
    ) -> Result<
        ArchivedRelationalBranch,
        RelationalBranchTransitionStop<RelationalBranchArchiveDenial>,
    > {
        let mut owner =
            self.owner
                .admitted_runtime()
                .ok_or(RelationalBranchTransitionStop::Denied(
                    RelationalBranchArchiveDenial::OwnerUnavailable,
                ))?;
        owner.archive_branch(execution, identity)
    }

    pub fn delete_branch(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &RelationalBranchIdentity,
    ) -> Result<
        RelationalBranchDeletionOutcome,
        RelationalBranchTransitionStop<RelationalBranchDeleteDenial>,
    > {
        let mut owner =
            self.owner
                .admitted_runtime()
                .ok_or(RelationalBranchTransitionStop::Denied(
                    RelationalBranchDeleteDenial::OwnerUnavailable,
                ))?;
        owner.delete_branch(execution, identity)
    }

    pub fn owner_lifecycle_observation(&self) -> RelationalOwnerLifecycleObservation {
        match self.owner.lifecycle_posture() {
            RelationalOwnerServiceLifecyclePosture::Open => {
                RelationalOwnerLifecycleObservation::Open
            }
            RelationalOwnerServiceLifecyclePosture::Closing => {
                RelationalOwnerLifecycleObservation::Closing
            }
            RelationalOwnerServiceLifecyclePosture::Closed => {
                RelationalOwnerLifecycleObservation::Closed
            }
        }
    }
}
