use super::super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationNoEffect, WorthQueryApplicationStaleAttempt,
    WorthQueryPendingApplicationCommitReceipt,
};
use crate::domain_computation::provider_session::WorthQueryMutationGraphWorkCompletion;

pub enum WorthQueryProviderProgressionOutcome {
    CoupledFailure(super::super::WorthQueryApplicationUncommitted),
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
        match self {
            Self::Committed(receipt) => Some(WorthQueryApplicationCommitOutcome::Committed(
                receipt.complete(completion)?,
            )),
            Self::AlreadyCommitted(receipt) => {
                Some(WorthQueryApplicationCommitOutcome::AlreadyCommitted(
                    receipt.with_retry_cleanup(completion)?,
                ))
            }
            uncommitted => match uncommitted.into_uncommitted() {
                Ok(outcome) => Some(outcome.into()),
                Err(_) => unreachable!("performed cases were handled with their actual completion"),
            },
        }
    }
    /// Move unpublished business evidence without issuing graph-work completion.
    pub(in crate::domain_computation::primary_graph) fn into_uncommitted(
        self,
    ) -> Result<super::super::WorthQueryApplicationUncommitted, Self> {
        use super::super::WorthQueryApplicationUncommitted as Uncommitted;
        Ok(match self {
            Self::CoupledFailure(outcome) => outcome,
            Self::ProductStale(stale) => Uncommitted::ProductStale(stale),
            Self::ProductUnpublished(value) => Uncommitted::ProductUnpublished(value),
            Self::NoEffect(value) => Uncommitted::NoEffect(value),
            Self::Stale(value) => Uncommitted::Stale(value),
            Self::Cancelled => Uncommitted::Cancelled,
            Self::TimedOut => Uncommitted::TimedOut,
            Self::Denied(value) => Uncommitted::Denied(value),
            Self::Aborted => Uncommitted::Aborted,
            Self::Deferred(value) => Uncommitted::Deferred(value),
            Self::Indeterminate(value) => Uncommitted::Indeterminate(value),
            published @ (Self::Committed(_) | Self::AlreadyCommitted(_)) => return Err(published),
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
        Kind::Stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
        )) => WorthQueryProviderProgressionOutcome::Cancelled,
        Kind::Stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
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
    use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind as Kind;
    match denial.kind() {
        Kind::Stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
        )) => WorthQueryApplicationCommitOutcome::Cancelled,
        Kind::Stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
        )) => WorthQueryApplicationCommitOutcome::TimedOut,
        _ => WorthQueryApplicationCommitOutcome::Denied(
            WorthQueryApplicationCommitDenial::request_authority_denied(stage, &denial),
        ),
    }
}
