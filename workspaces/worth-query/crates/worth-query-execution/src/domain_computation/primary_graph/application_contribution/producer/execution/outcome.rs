use super::denial::{denial, failed, handler_execution_failed, request_authority_stop};
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
pub(super) fn commit_receipt(
    identity: &str,
    outcome: WorthQueryApplicationCommitOutcome,
) -> Result<WorthQueryApplicationCommitReceipt, WorthQueryOutputDemandDenial> {
    // A request refused authorization at commit time stops as that request
    // does anywhere else; the producer did not fail.
    if let WorthQueryApplicationCommitOutcome::Denied(commit_denial) = &outcome {
        if let Some(authority) = commit_denial.request_authority() {
            return Err(request_authority_stop(identity, authority));
        }
    }
    // A companion with no room for this commit met a budget of the advance
    // that ran it; the producer did not fail, and its row stays claimable.
    if let Some(kind) = refused_budget(&outcome) {
        return Err(denial(kind, identity.to_owned()));
    }
    match outcome {
        WorthQueryApplicationCommitOutcome::Committed(receipt)
        | WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) => Ok(receipt),
        WorthQueryApplicationCommitOutcome::Deferred(deferred)
            if matches!(
                deferred.kind(),
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDeferredKind::RequiredPrerequisitePending(_)
            ) =>
        {
            Err(deferred.into_prerequisite_denial()
                .expect("required prerequisite deferral retains its actual denial"))
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

/// The budget a publication companion had no room in when it refused the
/// commit: the request's work, or what the index and its preparation retain.
fn refused_budget(
    outcome: &WorthQueryApplicationCommitOutcome,
) -> Option<WorthQueryOutputDemandDenialKind> {
    use crate::domain_computation::primary_graph::WorthQueryApplicationCommitDeferredKind as Deferred;
    use worth_relational::facade::mvcc::{
        CompanionPreflightStop as Stop, RelationalPublicationDeferred::CompanionPreflight,
    };
    let stop = match outcome {
        WorthQueryApplicationCommitOutcome::NoEffect(no_effect) => match no_effect.cause() {
            WorthQueryApplicationNoEffectCause::RelationalDeferred(CompanionPreflight(stop)) => {
                stop
            }
            _ => return None,
        },
        WorthQueryApplicationCommitOutcome::Deferred(deferred) => match deferred.kind() {
            Deferred::RelationalDeferred(CompanionPreflight(stop)) => stop,
            _ => return None,
        },
        _ => return None,
    };
    match stop {
        Stop::WorkExhausted { .. } | Stop::WorkCounterOverflow => {
            Some(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded)
        }
        Stop::PreparationMemoryExhausted { .. }
        | Stop::PreparationMemoryCounterOverflow
        | Stop::RetainedCompanionCapacityExhausted { .. } => {
            Some(WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::provider_session::{
        WorthQueryProviderSessionCommitDeferred, WorthQueryProviderSessionCommitDeferredKind,
    };
    use worth_relational::facade::mvcc::{
        CompanionPreflightStop as Stop, RelationalPublicationDeferred,
    };

    fn stopped(stop: Stop) -> WorthQueryOutputDemandDenialKind {
        let deferred = WorthQueryProviderSessionCommitDeferred::new(
            WorthQueryProviderSessionCommitDeferredKind::RelationalDeferred(
                RelationalPublicationDeferred::CompanionPreflight(stop),
            ),
            "",
        );
        commit_receipt(
            "producer",
            WorthQueryApplicationCommitOutcome::Deferred(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDeferred::from_provider_session(deferred),
            ),
        )
        .unwrap_err()
        .kind()
    }

    #[test]
    fn a_commit_a_companion_has_no_room_for_stops_for_that_budget() {
        use WorthQueryOutputDemandDenialKind as Kind;
        let retained = Stop::RetainedCompanionCapacityExhausted {
            requested: 2,
            retained: 1,
            maximum: 2,
        };
        assert_eq!(stopped(retained), Kind::RetentionBudgetExceeded);
        let prepared = Stop::PreparationMemoryExhausted {
            required: 2,
            maximum: 1,
        };
        assert_eq!(stopped(prepared), Kind::RetentionBudgetExceeded);
        let work = Stop::WorkExhausted {
            required: 2,
            maximum: 1,
        };
        assert_eq!(stopped(work), Kind::WorkBudgetExceeded);
        // A companion that is not ready is no budget of the advance.
        assert_eq!(stopped(Stop::TopologyPending), Kind::ProducerUnavailable);
    }
}
