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

impl std::fmt::Display for ProductBranchInventoryDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use worth_execution::report::Reportable as _;
        formatter.write_str(self.diagnostic_summary())
    }
}

impl std::error::Error for ProductBranchInventoryDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Branch(cause) => Some(cause),
            Self::Request(cause) => Some(cause),
            Self::AllocationRefused => None,
        }
    }
}

impl worth_execution::report::Reportable for ProductBranchInventoryDenial {
    fn denial_class(&self) -> worth_execution::report::DenialClass {
        match self {
            Self::Branch(cause) => cause.denial_class(),
            Self::Request(cause) => cause.denial_class(),
            Self::AllocationRefused => worth_execution::report::DenialClass::Exhausted,
        }
    }

    fn disposition(&self) -> worth_execution::report::Disposition {
        match self {
            Self::Branch(cause) => cause.disposition(),
            Self::Request(cause) => cause.disposition(),
            Self::AllocationRefused => worth_execution::report::Disposition::Final,
        }
    }

    fn report_code(&self) -> worth_execution::report::ReportCode {
        match self {
            Self::Branch(cause) => cause.report_code(),
            Self::Request(cause) => cause.report_code(),
            Self::AllocationRefused => worth_execution::report::ReportCode::new(
                "world.branch_inventory.allocation_refused",
            )
            .expect("static report code"),
        }
    }

    fn diagnostic_summary(&self) -> &'static str {
        match self {
            Self::Branch(_) => "A branch could not be admitted for inventory.",
            Self::Request(_) => "The execution request did not complete branch inventory.",
            Self::AllocationRefused => "The World could not allocate branch inventory.",
        }
    }
}
