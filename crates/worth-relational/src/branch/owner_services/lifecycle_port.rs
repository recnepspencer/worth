use super::owner_binding::{RelationalOwnerServiceBinding, RelationalOwnerServiceLifecyclePosture};
use crate::branch::{
    ArchivedRelationalBranch, RelationalBranchArchiveDenial, RelationalBranchDeleteDenial,
    RelationalBranchDeletionOutcome, RelationalBranchIdentity,
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
        identity: &RelationalBranchIdentity,
    ) -> Result<ArchivedRelationalBranch, RelationalBranchArchiveDenial> {
        let mut owner = self
            .owner
            .admitted_runtime()
            .ok_or(RelationalBranchArchiveDenial::OwnerUnavailable)?;
        owner.archive_branch(identity)
    }

    pub fn delete_branch(
        &self,
        identity: &RelationalBranchIdentity,
    ) -> Result<RelationalBranchDeletionOutcome, RelationalBranchDeleteDenial> {
        let mut owner = self
            .owner
            .admitted_runtime()
            .ok_or(RelationalBranchDeleteDenial::OwnerUnavailable)?;
        owner.delete_branch(identity)
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
