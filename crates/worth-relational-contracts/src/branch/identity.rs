use crate::history::data::coordinates::BranchId;

/// Typed denial for owner identity lookup. This is not a transaction-binding
/// denial and cannot admit a fork or transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBranchIdentityDenial {
    ForeignRuntime {
        expected_runtime_instance_id: u64,
        actual_runtime_instance_id: u64,
    },
    UnknownBranch(BranchId),
    IdentityMismatch,
}

/// Runtime-affine owner identity for a relational branch.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationalBranchIdentity {
    runtime_instance_id: u64,
    branch_id: BranchId,
}

impl RelationalBranchIdentity {
    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }
}

/// Operations used by the Relational owners.
pub trait RelationalBranchIdentityOwnerPort: Sized {
    fn new(runtime_instance_id: u64, branch_id: BranchId) -> Self;
    fn rebind(&self, runtime_instance_id: u64) -> Self;
}

impl RelationalBranchIdentityOwnerPort for RelationalBranchIdentity {
    fn new(runtime_instance_id: u64, branch_id: BranchId) -> Self {
        Self {
            runtime_instance_id,
            branch_id,
        }
    }

    fn rebind(&self, runtime_instance_id: u64) -> Self {
        Self::new(runtime_instance_id, self.branch_id.clone())
    }
}
