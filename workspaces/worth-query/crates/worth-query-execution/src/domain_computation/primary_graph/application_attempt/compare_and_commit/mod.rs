//! Compare-and-commit receipt and outcome surface.
mod cleanup_receipt;
mod cleanup_required;
pub use cleanup_receipt::{
    WorthQueryApplicationManagedCleanupReceipt, WorthQueryCleanupDispatchDenial,
    WorthQueryCleanupDispatchPosture,
};
pub use cleanup_required::{
    WorthQueryApplicationCleanupRequired, WorthQueryApplicationManagedCleanup,
};

mod commit_deferred;
pub(crate) mod commit_outcome;
mod commit_receipt;
mod committed_publication;

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
