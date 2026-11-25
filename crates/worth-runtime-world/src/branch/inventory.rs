//! A request-owned inventory over the installed World reference cells.
use super::{ProductBranchObservation, RuntimeWorldBranchAdmissionDenial};
use crate::identity::{
    CompositeCommitIdentity, ProductBranchIdentity, ProductBranchIncarnation,
    RuntimeWorldOwnerIdentity,
};

#[derive(Debug)]
pub enum ProductBranchInventoryDenial {
    Branch(RuntimeWorldBranchAdmissionDenial),
    Request(worth_execution::WorkCeilingDenial),
    AllocationRefused,
}

/// Descriptive metadata for one installed branch head; holds no observation slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledProductBranchMetadata {
    pub(crate) branch: ProductBranchIdentity,
    pub(crate) incarnation: ProductBranchIncarnation,
    pub(crate) head_commit: CompositeCommitIdentity,
}

impl InstalledProductBranchMetadata {
    pub const fn branch_identity(&self) -> &ProductBranchIdentity {
        &self.branch
    }

    pub const fn lifecycle_incarnation(&self) -> ProductBranchIncarnation {
        self.incarnation
    }

    pub const fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.incarnation.owner_identity()
    }

    pub const fn selected_commit(&self) -> &CompositeCommitIdentity {
        &self.head_commit
    }
}

/// Metadata-only inventory of installed branches; charges zero observation slots.
#[derive(Debug)]
pub struct InstalledProductBranches {
    pub(crate) branches: Vec<InstalledProductBranchMetadata>,
    pub(crate) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl InstalledProductBranches {
    pub fn branches(&self) -> &[InstalledProductBranchMetadata] {
        &self.branches
    }
}

/// Membership is captured once; heads are actual observations issued afterward.
/// This read does not claim an atomic cut across concurrently moving branches.
#[derive(Debug)]
pub struct ObservedProductBranches {
    pub(crate) branches: Vec<ProductBranchObservation>,
    pub(crate) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl ObservedProductBranches {
    pub fn branches(&self) -> &[ProductBranchObservation] {
        &self.branches
    }
}
