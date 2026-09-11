use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationIdempotencyResolutionDenial,
    WorthQueryApplicationIdempotencyResolutionDenialKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankEstateIdempotencyResolutionDenial {
    Authorization(crate::BankAuthorizationDenial),
    AuthorizationLineageUnavailable,
    ForeignAdmission,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    ProviderUnavailable,
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
    }
}
