//! Moved whole from `domain_computation::primary_graph::application_attempt::idempotency_resolution::denial`: the items below and every impl they own.
use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind;

/// Why an idempotency key could not be resolved. The resolution is a read, so
/// nothing took effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationIdempotencyResolutionDenialKind {
    /// Execution refused to finish the pending publication used by this read.
    ExecutionDenied(
        Result<
            crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind,
        >,
    ),
    /// The admission's current authority no longer holds, or inspecting the key was
    /// not authorized, for the named reason. The authorization denial has the
    /// contributing causes.
    Authorization(WorthQueryOperationAuthorizationDenialKind),
    /// The admission belongs to another runtime or schema binding, or has no product
    /// to resolve against.
    ForeignAdmission,
    /// The provider holds its maximum number of active snapshots.
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    /// The provider has no retention capacity for the read.
    RetentionCapacityExhausted,
    /// The provider has run out of retention identities.
    RetentionIdentityExhausted,
    /// The provider has run out of snapshot identities.
    SnapshotIdentityExhausted,
    /// The provider could not answer, the product was unpublished, or the
    /// recorded receipt could not be projected.
    ProviderUnavailable,
    /// The key is recorded with the same intent at `commit`, but this runtime
    /// no longer holds that commit's receipt: it was committed before a
    /// restore or reopen, or its product was retired. The commit took effect;
    /// retrying the same request cannot commit it again.
    CommittedReceiptNotRetained {
        commit: worth_relational::facade::history::CommitId,
    },
    /// The key's recorded intent was written by an earlier encoding whose
    /// operation part cannot be checked against this request. Neither the same
    /// intent nor drift is proven.
    RecordedIntentUnverifiable,
}
