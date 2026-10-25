#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchivedRelationalBranch {
    pub(super) identity: crate::branch::RelationalBranchIdentity,
    pub(super) observation: crate::branch::RelationalBranchReferenceObservation,
}

impl ArchivedRelationalBranch {
    pub fn identity(&self) -> &crate::branch::RelationalBranchIdentity {
        &self.identity
    }

    pub fn observation(&self) -> &crate::branch::RelationalBranchReferenceObservation {
        &self.observation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedRelationalBranch {
    pub(super) identity: crate::branch::RelationalBranchIdentity,
    pub(super) retired_root_identity: u64,
}

impl DeletedRelationalBranch {
    pub fn identity(&self) -> &crate::branch::RelationalBranchIdentity {
        &self.identity
    }

    pub const fn retired_root_identity(&self) -> u64 {
        self.retired_root_identity
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalBranchDeletionPending {
    pub(super) identity: crate::branch::RelationalBranchIdentity,
    pub(super) active_operation_count: u64,
}

impl RelationalBranchDeletionPending {
    pub fn identity(&self) -> &crate::branch::RelationalBranchIdentity {
        &self.identity
    }

    pub const fn active_operation_count(&self) -> u64 {
        self.active_operation_count
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBranchDeletionOutcome {
    Deleted(DeletedRelationalBranch),
    WaitingForActiveOperations(RelationalBranchDeletionPending),
}

impl RelationalBranchDeletionOutcome {
    pub fn deleted(&self) -> Option<&DeletedRelationalBranch> {
        match self {
            Self::Deleted(deleted) => Some(deleted),
            Self::WaitingForActiveOperations(_) => None,
        }
    }

    pub fn waiting(&self) -> Option<&RelationalBranchDeletionPending> {
        match self {
            Self::Deleted(_) => None,
            Self::WaitingForActiveOperations(waiting) => Some(waiting),
        }
    }
}
