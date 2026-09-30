use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationIdempotencyResolutionDenial,
    WorthQueryApplicationIdempotencyResolutionDenialKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankEstateIdempotencyResolutionDenial {
    Authorization(crate::BankAuthorizationDenial),
    AuthorizationLineageUnavailable,
    ForeignAdmission,
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
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
    match denial.kind() {
        WorthQueryApplicationIdempotencyResolutionDenialKind::Authorization => denial
            .authorization()
            .cloned()
            .map(crate::BankAuthorizationDenial::from_query)
            .map(BankEstateIdempotencyResolutionDenial::Authorization)
            .unwrap_or(BankEstateIdempotencyResolutionDenial::AuthorizationLineageUnavailable),
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
