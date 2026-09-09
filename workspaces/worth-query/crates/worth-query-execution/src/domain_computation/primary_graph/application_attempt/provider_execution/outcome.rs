use super::super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationStaleAttempt, WorthQueryPendingApplicationCommitReceipt,
};
use crate::domain_computation::provider_session::WorthQueryMutationGraphWorkCompletion;

pub(in crate::domain_computation) enum WorthQueryProviderProgressionOutcome {
    ProductStale(crate::domain_computation::WorthQueryProductStaleApplication),
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
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
