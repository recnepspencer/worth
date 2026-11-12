//! Typed causes retained across the recovery planning boundary.

use super::{PhysicalRecoveryPageAdmissionDenial, PhysicalRecoverySelectedReleaseHeadDenial};
use worth_store::physical_runtime::StoreRecoveryBindingSampleDenial;
use worth_store_recovery_physics::{
    OperationReconciliationDenial, PhysicalRedoPlanningDenial, RecoveryPlanCostDenial,
};

#[path = "ordered_release.rs"]
mod ordered_release;
pub use ordered_release::{
    PhysicalRecoveryOrderedReleaseDenial, PhysicalRecoveryOrderedReleaseJoin,
    PhysicalRecoveryOrderedReleaseStorage,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRecoveryPlanningDenial {
    ResolvedStateAllocation {
        requested_bytes: u64,
        cause: std::collections::TryReserveError,
    },
    PlannedStateAllocation {
        requested_bytes: u64,
        cause: std::collections::TryReserveError,
    },
    PublicationCandidateAllocation {
        requested_bytes: u64,
        cause: std::collections::TryReserveError,
    },
    ExecutionImageAllocation {
        requested_bytes: u64,
        cause: std::collections::TryReserveError,
    },
    BindingFreshness(StoreRecoveryBindingSampleDenial),
    BindingSamplingAllocation(
        worth_store::physical_runtime::StoreRecoveryBindingSampleAllocationDenial,
    ),
    OperationReconciliation(OperationReconciliationDenial),
    Redo(PhysicalRedoPlanningDenial),
    CustodyUnresolved,
    SelectedReleaseHead(PhysicalRecoverySelectedReleaseHeadDenial),
    ReleasedDirectorySource(super::PhysicalRecoverySelectedRecordReadDenial),
    ReleasedDirectoryProof(worth_store_recovery_physics::ReleasedDirectoryReplacementDenial),
    OrderedRelease(PhysicalRecoveryOrderedReleaseDenial),
    Page(PhysicalRecoveryPageAdmissionDenial),
    SuccessorCandidate(super::super::PhysicalRecoverySuccessorCandidateDenial),
    Cost(RecoveryPlanCostDenial),
}
#[cfg(test)]
#[path = "planning_denial_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
