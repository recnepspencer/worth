use super::denial::{denial, failed};
use super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};
use crate::domain_computation::primary_graph::{
    HandlerResult, WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationNoEffectCause,
};

/// A stable publication carries owner-issued lineage authority and cannot be
/// mistaken for a fresh commit receipt or a performed delivery obligation.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) enum ProducerExecutionOutcome
{
    Committed(WorthQueryApplicationCommitReceipt),
    Stable(crate::domain_computation::primary_graph::output_lineage::PublishedStableLineage),
}

/// A successful producer returns its original pre-effect Ready storage.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct PreparedProducerExecutionOutcome
{
    pub(super) outcome: ProducerExecutionOutcome,
    pub(super) ready_backing:
        crate::domain_computation::primary_graph::application_output_demand::PreparedReadyBacking,
}

impl PreparedProducerExecutionOutcome {
    pub(super) fn new(
        outcome: ProducerExecutionOutcome,
        ready_backing: crate::domain_computation::primary_graph::application_output_demand::PreparedReadyBacking,
    ) -> Self {
        Self {
            outcome,
            ready_backing,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn into_parts(
        self,
    ) -> (
        ProducerExecutionOutcome,
        crate::domain_computation::primary_graph::application_output_demand::PreparedReadyBacking,
    ) {
        (self.outcome, self.ready_backing)
    }
}

pub(super) fn completed_handler<Value, DomainDenial: std::fmt::Debug>(
    identity: &str,
    result: HandlerResult<Value, DomainDenial>,
) -> Result<Value, WorthQueryOutputDemandDenial> {
    match result {
        HandlerResult::Completed(value) => Ok(value),
        HandlerResult::DomainDenied(domain_denial) => Err(denial(
            WorthQueryOutputDemandDenialKind::ProducerUnavailable,
            format!("{identity}: producer domain denial: {domain_denial:?}"),
        )),
        HandlerResult::ExecutionDenied(error) => Err(failed(identity, error)),
        HandlerResult::Cancelled => Err(denial(
            WorthQueryOutputDemandDenialKind::Cancelled,
            "producer cancelled",
        )),
        HandlerResult::DeadlineExceeded => Err(denial(
            WorthQueryOutputDemandDenialKind::TimedOut,
            "producer deadline exceeded",
        )),
    }
}

/// Maps a producer commit to its receipt or to the demand denial it settles as.
pub(super) fn commit_receipt(
    identity: &str,
    outcome: WorthQueryApplicationCommitOutcome,
) -> Result<WorthQueryApplicationCommitReceipt, WorthQueryOutputDemandDenial> {
    match outcome {
        WorthQueryApplicationCommitOutcome::Committed(receipt)
        | WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) => Ok(receipt),
        WorthQueryApplicationCommitOutcome::Deferred(deferred)
            if matches!(
                deferred.kind(),
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDeferredKind::RequiredPrerequisitePending(_)
            ) =>
        {
            let crate::domain_computation::primary_graph::WorthQueryApplicationCommitDeferredKind::RequiredPrerequisitePending(kind) = deferred.kind() else {
                unreachable!("the guarded deferral names required prerequisite custody");
            };
            Err(denial(kind, identity.to_owned()).with_recovery_posture(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable,
            ))
        }
        WorthQueryApplicationCommitOutcome::Stale(_)
        | WorthQueryApplicationCommitOutcome::ProductStale(_) => Err(denial(
            WorthQueryOutputDemandDenialKind::PublicationStale,
            identity.to_owned(),
        )),
        WorthQueryApplicationCommitOutcome::Cancelled => Err(denial(
            WorthQueryOutputDemandDenialKind::Cancelled,
            identity.to_owned(),
        )),
        WorthQueryApplicationCommitOutcome::TimedOut => {
            Err(denial(WorthQueryOutputDemandDenialKind::TimedOut, identity.to_owned()))
        }
        WorthQueryApplicationCommitOutcome::NoEffect(no_effect)
            if no_effect.cause() == WorthQueryApplicationNoEffectCause::CapacityExhausted =>
        {
            Err(denial(
                WorthQueryOutputDemandDenialKind::PublicationCapacityExceeded,
                identity.to_owned(),
            ))
        }
        WorthQueryApplicationCommitOutcome::Denied(commit_denial)
            if commit_denial.kind() == WorthQueryApplicationCommitDenialKind::ProductBasisStale =>
        {
            Err(denial(
                WorthQueryOutputDemandDenialKind::PublicationStale,
                identity.to_owned(),
            ))
        }
        WorthQueryApplicationCommitOutcome::Denied(commit_denial)
            if commit_denial.kind()
                == WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift =>
        {
            Err(denial(
                WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                format!("{identity}: {commit_denial:?}"),
            ))
        }
        outcome => Err(failed(identity, outcome)),
    }
}
