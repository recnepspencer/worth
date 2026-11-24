pub mod cost_counters;
pub mod head_cell;
pub mod head_obligation;
pub mod head_retirement;
pub mod historical_retirement;
pub use historical_retirement::RelationalHistoricalObservationRetirement;
pub mod index_liveness;
pub mod lane;
#[cfg(test)]
pub(crate) use lane::assert_external_cut_and_lane_busy;
pub use lane::assert_recovered_cut_and_lane_busy_for_test;
#[doc(hidden)]
pub use lane::{RelationalRetentionCensusForTest, RelationalRetentionGateForTest};
pub mod lease;
pub mod obligation;
pub mod owner;
pub mod publication_basis;
pub mod publication_basis_tree;
pub use publication_basis::{PublicationBasisAccess, PublicationBasisImage, PublicationBasisRoute};
pub mod reclamation;
mod reclamation_court;
pub mod root_locators;
pub mod storage_inventory;
pub mod terminal_accounting;

pub use cost_counters::RelationalRetentionCostCounters;
pub use head_cell::RelationalBranchHeadRetentionCell;
pub use head_obligation::RelationalHeadRetentionObligation;
pub use head_retirement::{
    RelationalHeadRetirementCapacity, RelationalHeadRetirementReservation,
    TransferredHeadRetirement,
};
pub use lane::{
    RelationalBranchRetentionBinding, RelationalRetentionGuard,
    RelationalRetentionOwnerRelationship, RelationalRetentionRequestAcquisitionDenial,
    RelationalRetentionRetirementCustody, RelationalRetentionRetirementError,
};
pub use lease::RelationalExternalLeaseRetirement;
pub use lease::{
    RelationalBranchRetentionLease, RelationalBranchRetentionReleaseDenial,
    RelationalBranchRetentionReleaseReceipt, RelationalBranchRetentionTerminalOutcome,
};
pub use obligation::RelationalBasisRetentionReason;
pub use obligation::{
    RelationalCandidateRetentionObligation, RelationalExternalBasisRetentionObligation,
    RelationalObservationRetentionObligation, RelationalPerformedSettlementObligation,
    RelationalRetainedHistoricalRoot, RelationalRetentionObligationKind,
    RelationalTransactionRetentionObligation,
};
pub use owner::{
    CapturedExternalCheckpointComponents, RelationalBranchRetentionOwner,
    RelationalRetentionAcquisitionDenial,
};
pub use reclamation::RelationalBranchRootReclamationOutcome;
pub use reclamation::RetainedIndexRoots;
pub use storage_inventory::RelationalRetiredRootStorageSnapshot;
pub use terminal_accounting::RelationalExternalRetentionTerminalAccounting;
pub use terminal_accounting::RelationalExternalRetentionTerminalCounts;

mod capture_pause;
mod component_cut_tests;

pub use component_cut_tests::complete::arm_component_capture_for_test;

pub mod selected_snapshot;
pub mod selected_snapshot_custody;
pub mod snapshot_basis_retirement;
pub use selected_snapshot::{SnapshotHeadSelection, VisibilitySnapshotBasis};
pub use snapshot_basis_retirement::VisibilitySnapshotBasisRetirement;

/// Retention custody is temporarily busy. The same operation may be retried.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetentionBusy;
