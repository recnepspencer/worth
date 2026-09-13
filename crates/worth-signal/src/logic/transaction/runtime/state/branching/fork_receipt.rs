use crate::state::SignalBranchHandle;

#[cfg(test)]
use super::SignalBranchBasisArtifact;

#[derive(Debug, Clone)]
pub struct SignalBranchForkReceipt {
    #[cfg(test)]
    pub(super) parent_basis: SignalBranchBasisArtifact,
    #[cfg(test)]
    pub(super) requested_snapshot_basis: Option<SignalBranchBasisArtifact>,
    pub(super) created_branch: SignalBranchHandle,
    #[cfg(test)]
    pub(super) created_branch_basis: SignalBranchBasisArtifact,
    #[cfg(test)]
    pub(super) active_branch_after_fork_basis: SignalBranchBasisArtifact,
}

impl SignalBranchForkReceipt {
    #[cfg(test)]
    pub(crate) fn parent_basis(&self) -> &SignalBranchBasisArtifact {
        &self.parent_basis
    }

    #[cfg(test)]
    pub(crate) fn requested_snapshot_basis(&self) -> Option<&SignalBranchBasisArtifact> {
        self.requested_snapshot_basis.as_ref()
    }

    pub fn created_branch(&self) -> &SignalBranchHandle {
        &self.created_branch
    }

    #[cfg(test)]
    pub(crate) fn created_branch_basis(&self) -> &SignalBranchBasisArtifact {
        &self.created_branch_basis
    }

    #[cfg(test)]
    pub(crate) fn active_branch_after_fork_basis(&self) -> &SignalBranchBasisArtifact {
        &self.active_branch_after_fork_basis
    }
}
