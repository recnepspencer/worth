use crate::domain_computation::provider_session::WorthQueryMutationGraphWorkCompletion;
use {
    super::super::WorthQueryApplicationCommitDenial,
    super::super::WorthQueryApplicationCommitOutcome,
    super::super::WorthQueryApplicationCommitReceipt, super::super::WorthQueryApplicationNoEffect,
    super::super::WorthQueryPendingApplicationCommitReceipt,
    crate::domain_computation::primary_graph::vocabulary::application_attempt::commit_outcome::WorthQueryApplicationStaleAttempt,
    crate::domain_computation::primary_graph::vocabulary::application_attempt::denial::WorthQueryApplicationCommitDenialStage,
};

pub enum WorthQueryProviderProgressionOutcome {
    ProductStale(crate::domain_computation::WorthQueryProductStaleApplication),
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    NoEffect(WorthQueryApplicationNoEffect),
    Committed(WorthQueryPendingApplicationCommitReceipt),
    AlreadyCommitted(WorthQueryApplicationCommitReceipt),
    Stale(WorthQueryApplicationStaleAttempt),
    Cancelled,
    TimedOut,
    Denied(WorthQueryApplicationCommitDenial),
    Aborted,
    Deferred(super::super::WorthQueryApplicationCommitDeferred),
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
            Self::NoEffect(no_effect) => WorthQueryApplicationCommitOutcome::NoEffect(no_effect),
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
    use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind as Kind;
    match denial.kind() {
        Kind::Stopped(Err(
            crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::Cancelled,
        )) => WorthQueryProviderProgressionOutcome::Cancelled,
        Kind::Stopped(Err(
            crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::TimedOut,
        )) => WorthQueryProviderProgressionOutcome::TimedOut,
        _ => WorthQueryProviderProgressionOutcome::Denied(
            WorthQueryApplicationCommitDenial::request_authority_denied(stage, &denial),
        ),
    }
}

pub(in crate::domain_computation::primary_graph::application_attempt) fn commit_outcome_from_authorization_denial(
    denial: crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    stage: WorthQueryApplicationCommitDenialStage,
) -> WorthQueryApplicationCommitOutcome {
    use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind as Kind;
    match denial.kind() {
        Kind::Stopped(Err(
            crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::Cancelled,
        )) => WorthQueryApplicationCommitOutcome::Cancelled,
        Kind::Stopped(Err(
            crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::TimedOut,
        )) => WorthQueryApplicationCommitOutcome::TimedOut,
        _ => WorthQueryApplicationCommitOutcome::Denied(
            WorthQueryApplicationCommitDenial::request_authority_denied(stage, &denial),
        ),
    }
}
