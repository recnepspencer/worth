//! Pure idempotency resolution outcome enums.
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationIdempotencyResolutionDenial,
};

#[derive(Debug)]
pub enum WorthQueryApplicationIdempotencyResolution {
    ReleasedOriginal(crate::domain_computation::primary_graph::WorthQueryReleasedApplicationEffect),
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    ProductPreparationPending(crate::domain_computation::WorthQueryProductPreparationPending),
    /// No commit is recorded under this key.
    Unseen,
    RecoveredCommitted(crate::domain_computation::primary_graph::provider::WorthQueryRecoveredApplicationCommitReceipt),
    /// The key was already used for a different intent.
    IntentDrift,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGuardedWorkflowOperationCustody {
    ReleasedOriginal(crate::domain_computation::primary_graph::WorthQueryReleasedApplicationEffect),
    Unseen,
    Committed(WorthQueryApplicationCommitReceipt),
    RecoveredCommitted(crate::domain_computation::primary_graph::provider::WorthQueryRecoveredApplicationCommitReceipt),
    ExternallySettled(WorthQueryApplicationCommitReceipt),
    DispatchPending(WorthQueryApplicationCommitReceipt),
    IntentDrift,
    PublicationPending,
    ProductUnpublished(worth_runtime_world::facade::ProductUnpublishedRecoveryHandle),
    Indeterminate(WorthQueryApplicationIdempotencyResolutionDenial),
}
