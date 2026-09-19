//! Allocation truth progression: candidate -> preview or committed receipt.

#[path = "report_freshness/allocation_counters.rs"]
mod allocation_counters;
#[path = "committed_truth/candidate.rs"]
mod candidate;
#[path = "transaction/commit_outcome.rs"]
mod commit_outcome;
#[path = "committed_truth/committed_allocation.rs"]
mod committed_allocation;
#[path = "committed_truth/committed_catalog_binding.rs"]
mod committed_catalog_binding;
#[path = "committed_truth/committed_evidence.rs"]
mod committed_evidence;
#[path = "committed_truth/committed_lowering_input.rs"]
mod committed_lowering_input;
#[path = "committed_truth/committed_receipt.rs"]
mod committed_receipt;
#[path = "ledger_lifecycle/completed_replay.rs"]
mod completed_replay;
#[path = "report_freshness/consumer_admission.rs"]
mod consumer_admission;
#[path = "transaction/denial.rs"]
mod denial;
#[path = "transaction/denial_report.rs"]
mod denial_report;
#[path = "transaction/denial_taxonomy.rs"]
mod denial_taxonomy;
#[path = "committed_truth/durable_semantic_state.rs"]
mod durable_semantic_state;
#[path = "committed_truth/equivalence_basis.rs"]
mod equivalence_basis;
#[path = "committed_truth/geometry_evidence.rs"]
mod geometry_evidence;
#[path = "ledger_lifecycle/ledger_denial.rs"]
mod ledger_denial;
#[path = "ledger_lifecycle/ledger_state.rs"]
mod ledger_state;
#[path = "ledger_lifecycle/mounted_projection/catalog.rs"]
mod mounted_projection_catalog;
#[path = "ledger_lifecycle/mounted_projection/journal.rs"]
mod mounted_projection_journal;
#[path = "ledger_lifecycle/mounted_projection/overlay_extent_export.rs"]
mod mounted_projection_overlay_extent_export;
#[path = "ledger_lifecycle/mounted_projection/row.rs"]
mod mounted_projection_row;
#[path = "transaction/prepared_portal_commit.rs"]
mod prepared_portal_commit;
#[path = "transaction/prepared_replan.rs"]
mod prepared_replan;
#[path = "reuse/preview_candidate.rs"]
mod preview_candidate;
#[path = "reuse/preview_isolation.rs"]
mod preview_isolation;
#[path = "reuse/receipt_budget.rs"]
mod receipt_budget;
#[path = "transaction/receipt_commit.rs"]
mod receipt_commit;
#[path = "report_freshness/receipt_generation.rs"]
mod receipt_generation;
#[path = "report_freshness/receipt_identity.rs"]
mod receipt_identity;
#[path = "ledger_lifecycle/receipt_ledger.rs"]
mod receipt_ledger;
#[path = "ledger_lifecycle/receipt_ledger_entry.rs"]
mod receipt_ledger_entry;
#[cfg(test)]
#[path = "ledger_lifecycle/receipt_ledger_test_support.rs"]
mod receipt_ledger_test_support;
#[cfg(test)]
pub(crate) use receipt_ledger_test_support::{
    detached_non_portal_receipt, UiNonPortalReceiptLawCandidate,
};
#[cfg(test)]
mod allocation_contract_tests;
#[cfg(test)]
mod hostile_workbench_tests;
#[path = "report_freshness/receipt_report.rs"]
mod receipt_report;
#[path = "transaction/replan_commit_mode.rs"]
mod replan_commit_mode;
#[path = "transaction/replan_transaction.rs"]
mod replan_transaction;
#[path = "reuse/reuse_verdict.rs"]
mod reuse_verdict;
#[path = "transaction/transaction_outcome.rs"]
mod transaction_outcome;
#[path = "committed_truth/truth_revision.rs"]
mod truth_revision;
#[path = "report_freshness/viewport_inspection.rs"]
mod viewport_inspection;

pub use allocation_counters::{UiAllocationCounterName, UiAllocationCounterReport};
pub use candidate::UiAllocationCandidate;
pub use commit_outcome::UiAllocationReceiptCommitOutcome;
pub(crate) use committed_allocation::UiCommittedAllocation;
pub(crate) use committed_catalog_binding::UiCommittedAllocationCatalogActivation;
pub use committed_catalog_binding::UiCommittedAllocationCatalogActivationDenial;
pub(crate) use committed_catalog_binding::UiCommittedAllocationCatalogActivationRow;
pub(crate) use committed_catalog_binding::UiCommittedAllocationCatalogBindings;
pub(crate) use committed_catalog_binding::UiCommittedPortalActivationSource;
pub(crate) use committed_catalog_binding::UiCommittedScrollActivationSource;
pub use committed_evidence::UiCommittedAllocationEvidenceSet;
pub use committed_lowering_input::UiCommittedAllocationLoweringInput;
pub use committed_receipt::UiAllocationReceipt;
#[cfg(test)]
pub use consumer_admission::admit_host_paint;
pub use consumer_admission::{admit_execution_lowering, UiAllocationFreshnessConsumptionDenial};
pub use denial::UiAllocationReceiptCommitDenial;
pub use denial_report::UiAllocationReceiptDenialReport;
pub use denial_taxonomy::UiAllocationDenialFamily;
pub use durable_semantic_state::UiAllocationDurableSemanticState;
pub use equivalence_basis::UiAllocationReceiptEquivalenceBasis;
pub use geometry_evidence::{
    UiAllocationAnchorPosture, UiAllocationAxis, UiAllocationAxisAlignedBounds,
    UiAllocationEdgeReference, UiAllocationGeometryKnowledge,
    UiCommittedAllocationGeometryEvidence,
};
pub use prepared_portal_commit::UiPortalAllocationCommitBindDenial;
pub use preview_candidate::UiAllocationPreviewCandidate;
pub use preview_isolation::{UiPreviewPaintIsolationOutcome, UiPreviewPaintIsolationViolation};
pub use receipt_generation::UiAllocationReceiptGeneration;
pub use receipt_identity::UiAllocationReceiptIdentity;
#[cfg(test)]
pub use receipt_report::{
    UiAllocationFreshnessTransitionCause, UiAllocationFreshnessTransitionDenial,
};
pub use receipt_report::{
    UiAllocationReceiptFreshnessPosture, UiAllocationReceiptLagBound, UiAllocationReceiptReport,
};
pub use replan_transaction::UiAllocationReplanTransaction;
pub use reuse_verdict::{
    UiAllocationLeafRemeasureWitness, UiAllocationReuseDenial, UiAllocationReuseVerdict,
};
pub use transaction_outcome::{
    UiAllocationAuthoritySuccessionDenial, UiAllocationReplanTransactionCommitDenial,
    UiAllocationReplanTransactionCounters, UiAllocationReplanTransactionOutcome,
    UiCommittedAllocationReplan,
};
#[cfg(test)]
pub use truth_revision::UiAllocationAuthorityCounter;
pub use truth_revision::{
    UiAllocationAuthorityCounterExhaustion, UiAllocationTruthDelta, UiAllocationTruthRevision,
};

pub(crate) use ledger_state::UiAllocationCatalogLedgerLineage;
pub(crate) use ledger_state::UiAllocationCatalogLedgerTransition;
pub(crate) use mounted_projection_catalog::UiMountedAllocationProjectionCatalog;
pub(crate) use mounted_projection_journal::{
    UiMountedAllocationExactDelta, UiMountedAllocationProjectionDelta,
    UiMountedAllocationProjectionSource,
};
#[allow(
    unused_imports,
    reason = "Gate 1 retains sealed mounted overlay extent denials for later publication"
)]
pub(crate) use mounted_projection_overlay_extent_export::{
    UiCommittedOverlayExtentBounds, UiMountedOverlayExtentDenial, UiMountedOverlayExtentOwner,
    UiMountedOverlayExtentOwnerExport, UiMountedOverlayRegionExtent,
};
pub(crate) use mounted_projection_row::{
    UiCommittedViewportGeometry, UiMountedAllocationProjectionDenial,
};
pub(in crate::runtime) use prepared_replan::{
    UiAllocationLedgerPreparation, UiPreparedAllocationLedgerTransition,
};
pub(crate) use preview_isolation::UiPreviewPaintIsolationPort;
pub(crate) use receipt_commit::project_allocation_preview;
pub(crate) use receipt_ledger::UiAllocationReceiptLedger;
pub(in crate::runtime) use receipt_ledger_entry::UiPreparedAllocationCatalogLedgerCommit;
#[path = "ledger_lifecycle/activation_catalog_commit.rs"]
mod activation_catalog_commit;
#[path = "ledger_lifecycle/removal_catalog_commit.rs"]
mod removal_catalog_commit;
