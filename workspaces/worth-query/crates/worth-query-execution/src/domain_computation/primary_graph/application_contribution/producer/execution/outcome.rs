mod commit_denial;
use super::super::demand::ProducerPublicationStale;
use super::denial::{
    commit_execution_stopped, denial, failed, handler_execution_failed, request_authority_stop,
};
use super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};
use crate::domain_computation::primary_graph as graph;
use graph::{
    HandlerResult, WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationNoEffectCause,
};

/// A stable publication carries owner-issued lineage authority and cannot be
/// mistaken for a fresh commit receipt or a performed delivery obligation.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) enum ProducerExecutionOutcome
{
    Committed(ProducerCommitReceipt),
    Stable(graph::output_lineage::PublishedStableLineage),
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer) type PreparedProducerExecutionPlan<
    'owner,
> = super::prepared_plan::OwnerPlan<
    'owner,
    PreparedProducerExecutionOutcome,
    super::ProducerExecutionStop,
>;
/// The commit owner distinguishes a new effect from a descriptive replay.
/// Only this conversion constructs the fact used to mark the advance.
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct ProducerCommitReceipt
{
    receipt: WorthQueryApplicationCommitReceipt,
    publication: CommitPublication,
}

#[derive(Debug)]
enum CommitPublication {
    PublishedNow,
    PreviouslyPublished,
}

impl ProducerCommitReceipt {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn published_in_this_advance(
        &self,
    ) -> bool {
        matches!(self.publication, CommitPublication::PublishedNow)
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn record_publication(
        self,
        record: impl FnOnce(),
    ) -> Self {
        if self.published_in_this_advance() {
            record();
        }
        self
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn into_receipt(
        self,
    ) -> WorthQueryApplicationCommitReceipt {
        self.receipt
    }
}

#[cfg(test)]
mod publication_record_tests;

/// A successful producer returns its original pre-effect Ready storage.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct PreparedProducerExecutionOutcome
{
    pub(super) outcome: ProducerExecutionOutcome,
    pub(super) ready_backing: graph::application_output_demand::PreparedReadyBacking,
}

impl PreparedProducerExecutionOutcome {
    pub(super) fn new(
        outcome: ProducerExecutionOutcome,
        ready_backing: graph::application_output_demand::PreparedReadyBacking,
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
        graph::application_output_demand::PreparedReadyBacking,
    ) {
        (self.outcome, self.ready_backing)
    }
}

pub(super) fn completed_handler<Value, DomainDenial>(
    identity: &'static str,
    result: HandlerResult<Value, DomainDenial>,
    domain_reason: impl FnOnce(&DomainDenial) -> Option<&'static str>,
) -> Result<Value, WorthQueryOutputDemandDenial> {
    match result {
        HandlerResult::Completed(value) => Ok(value),
        HandlerResult::DomainDenied(domain_denial) => {
            Err(WorthQueryOutputDemandDenial::producer_domain_denied(
                identity,
                domain_reason(&domain_denial),
            ))
        }
        HandlerResult::ExecutionDenied(error) => Err(handler_execution_failed(identity, error)),
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
pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn commit_receipt(
    identity: &str,
    outcome: WorthQueryApplicationCommitOutcome,
) -> Result<ProducerCommitReceipt, WorthQueryOutputDemandDenial> {
    // A request refused authorization at commit time stops as that request
    // does anywhere else; the producer did not fail.
    if let WorthQueryApplicationCommitOutcome::Denied(commit_denial) = &outcome
        && let Some(authority) = commit_denial.authorization_denial_kind()
    {
        return Err(request_authority_stop(identity, authority));
    }
    // A stopped native companion preflight belongs to this advance. A budget
    // refusal or replaced selected image leaves the original producer claimable.
    if let Some(refused) = refused_preflight(identity, &outcome) {
        return Err(refused);
    }
    match outcome {
        WorthQueryApplicationCommitOutcome::CleanupRequired(cleanup) => {
            let denial = WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ManagedCleanupRequired(
                    cleanup.cleanup().failure().kind(),
                ),
                identity.to_owned(),
            );
            Err(denial.with_managed_cleanup(cleanup))
        }
        WorthQueryApplicationCommitOutcome::Committed(receipt) => Ok(ProducerCommitReceipt {
            receipt,
            publication: CommitPublication::PublishedNow,
        }),
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) => {
            Ok(ProducerCommitReceipt {
                receipt,
                publication: CommitPublication::PreviouslyPublished,
            })
        }
        WorthQueryApplicationCommitOutcome::Deferred(deferred) => {
            match deferred.into_prerequisite_denial() {
                Ok(denial) => Err(denial),
                Err(deferred) => Err(failed(
                    identity,
                    WorthQueryApplicationCommitOutcome::Deferred(deferred),
                )),
            }
        }
        WorthQueryApplicationCommitOutcome::Stale(stale) => Err(denial(
            WorthQueryOutputDemandDenialKind::PublicationStale,
            identity.to_owned(),
        )
        .with_publication_stale(Some(ProducerPublicationStale::CommitDecisionFacts(stale)))),
        WorthQueryApplicationCommitOutcome::ProductStale(stale) => Err(denial(
            WorthQueryOutputDemandDenialKind::PublicationStale,
            identity.to_owned(),
        )
        .with_publication_stale(Some(ProducerPublicationStale::product_head(&stale)))),
        WorthQueryApplicationCommitOutcome::Cancelled => Err(denial(
            WorthQueryOutputDemandDenialKind::Cancelled,
            identity.to_owned(),
        )),
        WorthQueryApplicationCommitOutcome::TimedOut => Err(denial(
            WorthQueryOutputDemandDenialKind::TimedOut,
            identity.to_owned(),
        )),
        WorthQueryApplicationCommitOutcome::NoEffect(no_effect)
            if no_effect.cause() == WorthQueryApplicationNoEffectCause::CapacityExhausted =>
        {
            Err(denial(
                WorthQueryOutputDemandDenialKind::PublicationCapacityExceeded,
                identity.to_owned(),
            ))
        }
        WorthQueryApplicationCommitOutcome::Denied(commit_denial) => {
            let kind = commit_denial.kind();
            let capacity = commit_denial.output_verification_capacity_stop();
            Err(commit_denial::stopped(
                identity,
                kind,
                commit_denial.stage(),
                commit_denial.execution_denial_cause(),
                capacity,
                &commit_denial,
            ))
        }
        WorthQueryApplicationCommitOutcome::ProductUnpublished(outcome) => Err(denial(
            WorthQueryOutputDemandDenialKind::ProducerUnavailable,
            identity.to_owned(),
        )
        .with_uncommitted(graph::WorthQueryApplicationUncommitted::ProductUnpublished(
            outcome,
        ))),
        WorthQueryApplicationCommitOutcome::Indeterminate(outcome) => Err(denial(
            WorthQueryOutputDemandDenialKind::ProducerUnavailable,
            identity.to_owned(),
        )
        .with_uncommitted(graph::WorthQueryApplicationUncommitted::Indeterminate(
            outcome,
        ))),
        outcome @ (WorthQueryApplicationCommitOutcome::NoEffect(_)
        | WorthQueryApplicationCommitOutcome::Aborted) => Err(failed(identity, outcome)),
    }
}

/// Native preflight refuses this advance before effects. Preserve original
/// budgets and a replaced selected image as retryable owner outcomes.
fn refused_preflight(
    identity: &str,
    outcome: &WorthQueryApplicationCommitOutcome,
) -> Option<WorthQueryOutputDemandDenial> {
    use graph::WorthQueryApplicationCommitDeferredKind as Deferred;
    use worth_relational::facade::mvcc::{
        CompanionPreflightStop as Stop, RelationalPublicationDeferred::CompanionPreflight,
    };
    let (stop, origin) = match outcome {
        WorthQueryApplicationCommitOutcome::NoEffect(no_effect) => match no_effect.cause() {
            WorthQueryApplicationNoEffectCause::RelationalDeferred(CompanionPreflight(stop)) => (
                stop,
                ProducerPublicationStale::CommitNoEffectPreflight(stop),
            ),
            _ => return None,
        },
        WorthQueryApplicationCommitOutcome::Deferred(deferred) => match deferred.kind() {
            Deferred::RelationalDeferred(CompanionPreflight(stop)) => (
                stop,
                ProducerPublicationStale::CommitDeferredPreflight(stop),
            ),
            _ => return None,
        },
        WorthQueryApplicationCommitOutcome::CleanupRequired(_)
        | WorthQueryApplicationCommitOutcome::Committed(_)
        | WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
        | WorthQueryApplicationCommitOutcome::ProductStale(_)
        | WorthQueryApplicationCommitOutcome::ProductUnpublished(_)
        | WorthQueryApplicationCommitOutcome::Stale(_)
        | WorthQueryApplicationCommitOutcome::Cancelled
        | WorthQueryApplicationCommitOutcome::TimedOut
        | WorthQueryApplicationCommitOutcome::Denied(_)
        | WorthQueryApplicationCommitOutcome::Aborted
        | WorthQueryApplicationCommitOutcome::Indeterminate(_) => return None,
    };
    let kind = match stop {
        Stop::SelectedSourceMismatch => Some(WorthQueryOutputDemandDenialKind::PublicationStale),
        Stop::WorkExhausted { .. }
        | Stop::WorkExhaustedWithoutRequirement { .. }
        | Stop::WorkCounterOverflow => Some(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded),
        Stop::PreparationMemoryExhausted { .. }
        | Stop::PreparationMemoryCounterOverflow
        | Stop::RetainedCompanionCapacityExhausted { .. } => {
            Some(WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded)
        }
        _ => None,
    }?;
    let stale = (stop == Stop::SelectedSourceMismatch).then_some(origin);
    Some(
        denial(kind, identity.to_owned())
            .with_publication_stale(stale)
            .with_publication_companion_stop(stop),
    )
}

#[cfg(test)]
mod tests;
