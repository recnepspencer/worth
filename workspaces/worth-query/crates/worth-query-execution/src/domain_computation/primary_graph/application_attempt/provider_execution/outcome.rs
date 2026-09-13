use super::super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationNoEffect, WorthQueryApplicationStaleAttempt,
    WorthQueryPendingApplicationCommitReceipt,
};
use crate::domain_computation::provider_session::WorthQueryMutationGraphWorkCompletion;

pub(in crate::domain_computation) enum WorthQueryProviderProgressionOutcome {
    ProductStale(crate::domain_computation::WorthQueryProductStaleApplication),
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    NoEffect(worth_runtime_world::facade::NoEffectCompositePublication),
    Committed(WorthQueryPendingApplicationCommitReceipt),
    AlreadyCommitted(WorthQueryApplicationCommitReceipt),
    Stale(WorthQueryApplicationStaleAttempt),
    Cancelled,
    TimedOut,
    Denied(WorthQueryApplicationCommitDenial),
    Aborted,
    Deferred(super::super::WorthQueryApplicationCommitDeferred),
    SettlementDeferred(super::super::WorthQueryApplicationSettlementDeferred),
    Indeterminate(super::super::WorthQueryApplicationUnresolvedCommitEvidence),
}

impl WorthQueryProviderProgressionOutcome {
    pub(super) fn finish(
        self,
        completion: WorthQueryMutationGraphWorkCompletion,
    ) -> Option<WorthQueryApplicationCommitOutcome> {
        Some(match self {
            Self::ProductStale(stale) => WorthQueryApplicationCommitOutcome::ProductStale(stale),
            Self::ProductUnpublished(unpublished) => {
                WorthQueryApplicationCommitOutcome::ProductUnpublished(unpublished)
            }
            Self::NoEffect(no_effect) => WorthQueryApplicationCommitOutcome::NoEffect(
                WorthQueryApplicationNoEffect::from_world(no_effect),
            ),
            Self::Committed(receipt) => {
                WorthQueryApplicationCommitOutcome::Committed(receipt.complete(completion)?)
            }
            Self::AlreadyCommitted(receipt) => {
                WorthQueryApplicationCommitOutcome::AlreadyCommitted(
                    receipt.with_retry_cleanup(completion)?,
                )
            }
            Self::Stale(stale) => WorthQueryApplicationCommitOutcome::Stale(stale),
            Self::Cancelled => WorthQueryApplicationCommitOutcome::Cancelled,
            Self::TimedOut => WorthQueryApplicationCommitOutcome::TimedOut,
            Self::Denied(denial) => WorthQueryApplicationCommitOutcome::Denied(denial),
            Self::Aborted => WorthQueryApplicationCommitOutcome::Aborted,
            Self::Deferred(deferred) => WorthQueryApplicationCommitOutcome::Deferred(deferred),
            Self::SettlementDeferred(deferred) => {
                WorthQueryApplicationCommitOutcome::SettlementDeferred(deferred)
            }
            Self::Indeterminate(evidence) => {
                WorthQueryApplicationCommitOutcome::Indeterminate(evidence)
            }
        })
    }
}

pub(in crate::domain_computation) fn progression_denied(
    stage: WorthQueryApplicationCommitDenialStage,
) -> WorthQueryProviderProgressionOutcome {
    WorthQueryProviderProgressionOutcome::Denied(
        WorthQueryApplicationCommitDenial::provider_rejected(stage),
    )
}

pub(in crate::domain_computation::primary_graph::application_attempt) fn progression_from_authorization_denial(
    denial: crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    stage: WorthQueryApplicationCommitDenialStage,
) -> WorthQueryProviderProgressionOutcome {
    use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind as Kind;
    match denial.kind() {
        Kind::Cancelled => WorthQueryProviderProgressionOutcome::Cancelled,
        Kind::DeadlineExceeded => WorthQueryProviderProgressionOutcome::TimedOut,
        _ => progression_denied(stage),
    }
}

pub(in crate::domain_computation::primary_graph::application_attempt) fn commit_outcome_from_authorization_denial(
    denial: crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    stage: WorthQueryApplicationCommitDenialStage,
) -> WorthQueryApplicationCommitOutcome {
    use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind as Kind;
    match denial.kind() {
        Kind::Cancelled => WorthQueryApplicationCommitOutcome::Cancelled,
        Kind::DeadlineExceeded => WorthQueryApplicationCommitOutcome::TimedOut,
        _ => WorthQueryApplicationCommitOutcome::Denied(
            WorthQueryApplicationCommitDenial::provider_rejected(stage),
        ),
    }
}
