//! Compare-and-commit receipt and outcome surface.
use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_attempt::compare_and_commit::cleanup_receipt;
use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_attempt::compare_and_commit::cleanup_required;
pub use cleanup_receipt::{
    WorthQueryApplicationManagedCleanupReceipt, WorthQueryCleanupDispatchDenial,
    WorthQueryCleanupDispatchPosture,
};
pub use cleanup_required::{
    WorthQueryApplicationCleanupRequired, WorthQueryApplicationManagedCleanup,
};

use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_attempt::compare_and_commit::commit_deferred;
pub(crate) use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_attempt::compare_and_commit::commit_outcome;
mod commit_receipt;
use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_attempt::compare_and_commit::committed_publication;

pub use commit_deferred::{
    WorthQueryApplicationCommitDeferred, WorthQueryApplicationCommitDeferredKind,
};
pub(in crate::domain_computation::primary_graph) use commit_outcome::WorthQueryProgramActivationUnresolved;
pub use commit_outcome::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitRecoveryKind, WorthQueryApplicationLandedCommit,
    WorthQueryApplicationManagedCleanupFailure, WorthQueryApplicationNoEffect,
    WorthQueryApplicationNoEffectCause, WorthQueryApplicationStaleAttempt,
    WorthQueryApplicationUncommitted, WorthQueryApplicationUnresolvedCommitEvidence,
};
pub(in crate::domain_computation::primary_graph) use commit_receipt::WorthQueryCommittedReceiptProjection;
pub use commit_receipt::{
    WorthQueryApplicationCommitPublicationSource, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationCommittedChanges,
};
pub use committed_publication::WorthQueryCommittedProductPublication;

pub(in crate::domain_computation::primary_graph) use commit_receipt::MaterializedCommittedField;

pub use commit_receipt::WorthQueryPendingApplicationCommitReceipt;

pub(in crate::domain_computation) use cleanup_receipt::RetainedOriginalCleanupCommit;
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
pub(in crate::domain_computation::primary_graph) use commit_receipt::recording;
