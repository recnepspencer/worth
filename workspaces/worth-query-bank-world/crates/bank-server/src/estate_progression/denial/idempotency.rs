use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationIdempotencyResolutionDenial,
    WorthQueryApplicationIdempotencyResolutionDenialKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankEstateIdempotencyResolutionDenial {
    Authorization(crate::BankAuthorizationDenial),
    ForeignAdmission,
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    IdempotencyWindowExpired,
    ProviderUnavailable,
    /// The key is recorded with the same intent and that commit took effect,
    /// but the runtime no longer holds its receipt, as after a restore.
    CommittedReceiptNotRetained,
    /// The key's recorded intent was written by an earlier encoding that
    /// cannot be checked against this request.
    RecordedIntentUnverifiable,
}

pub(super) fn from_query(
    denial: WorthQueryApplicationIdempotencyResolutionDenial,
) -> BankEstateIdempotencyResolutionDenial {
    from_kind(
        denial.kind(),
        denial
            .authorization()
            .map_or(0, |authorization| authorization.causes().len()),
    )
}

/// Each resolution cause keeps its own Bank cause; an authorization refusal
/// keeps its exact kind and how many causes contributed to it.
fn from_kind(
    kind: WorthQueryApplicationIdempotencyResolutionDenialKind,
    contributing_cause_count: usize,
) -> BankEstateIdempotencyResolutionDenial {
    match kind {
        WorthQueryApplicationIdempotencyResolutionDenialKind::Authorization(kind) => {
            BankEstateIdempotencyResolutionDenial::Authorization(
                crate::BankAuthorizationDenial::from_kind(kind, contributing_cause_count),
            )
        }
        WorthQueryApplicationIdempotencyResolutionDenialKind::ForeignAdmission => {
            BankEstateIdempotencyResolutionDenial::ForeignAdmission
        }
        WorthQueryApplicationIdempotencyResolutionDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => BankEstateIdempotencyResolutionDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        WorthQueryApplicationIdempotencyResolutionDenialKind::RetentionCapacityExhausted => {
            BankEstateIdempotencyResolutionDenial::RetentionCapacityExhausted
        }
        WorthQueryApplicationIdempotencyResolutionDenialKind::RetentionIdentityExhausted => {
            BankEstateIdempotencyResolutionDenial::RetentionIdentityExhausted
        }
        WorthQueryApplicationIdempotencyResolutionDenialKind::SnapshotIdentityExhausted => {
            BankEstateIdempotencyResolutionDenial::SnapshotIdentityExhausted
        }
        WorthQueryApplicationIdempotencyResolutionDenialKind::IdempotencyWindowExpired => {
            BankEstateIdempotencyResolutionDenial::IdempotencyWindowExpired
        }
        WorthQueryApplicationIdempotencyResolutionDenialKind::ProviderUnavailable => {
            BankEstateIdempotencyResolutionDenial::ProviderUnavailable
        }
        WorthQueryApplicationIdempotencyResolutionDenialKind::CommittedReceiptNotRetained {
            ..
        } => BankEstateIdempotencyResolutionDenial::CommittedReceiptNotRetained,
        WorthQueryApplicationIdempotencyResolutionDenialKind::RecordedIntentUnverifiable => {
            BankEstateIdempotencyResolutionDenial::RecordedIntentUnverifiable
        }
    }
}

#[cfg(test)]
#[path = "idempotency/kind_tests.rs"]
mod kind_tests;
