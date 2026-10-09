use super::denial::{denial, failed, handler_execution_failed, request_authority_stop};
use super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};
use crate::domain_computation::primary_graph as graph;
use graph::{
    HandlerResult, WorthQueryApplicationCommitDeferredKind as DeferredKind,
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationNoEffectCause,
};

/// A stable publication carries owner-issued lineage authority and cannot be
/// mistaken for a fresh commit receipt or a performed delivery obligation.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) enum ProducerExecutionOutcome
{
    Committed(WorthQueryApplicationCommitReceipt),
    Stable(graph::output_lineage::PublishedStableLineage),
}

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
                DeferredKind::RequiredPrerequisitePending(_)
            ) =>
        {
            Err(deferred
                .into_prerequisite_denial()
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
            use WorthQueryApplicationCommitDenialKind as Kind;
            let kind = commit_denial.kind();
            let execution = commit_denial
                .execution_denial_cause()
                .map(|cause| (commit_denial.stage(), cause));
            let failure = match kind {
                Kind::ProductBasisStale => denial(
                    WorthQueryOutputDemandDenialKind::PublicationStale,
                    identity.to_owned(),
                ),
                Kind::IdempotencyIntentDrift => denial(
                    WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                    format!("{identity}: {commit_denial:?}"),
                ),
                Kind::ExecutionResource { .. }
                | Kind::ExecutionNestedPatternStopped { .. }
                | Kind::ExecutionWorkerPanicked { .. }
                | Kind::ExecutionUncheckedCustomKernel { .. }
                | Kind::ExecutionIdentitiesNotCanonical { .. }
                | Kind::ProviderRejected
                | Kind::CustomInvariantDenied
                | Kind::WorkflowSettlementDenied { .. }
                | Kind::UniqueValueTaken
                | Kind::UniqueIndexUnavailable
                | Kind::ActiveSnapshotCapacityExhausted { .. }
                | Kind::RetentionCapacityExhausted
                | Kind::RetentionIdentityExhausted
                | Kind::SnapshotIdentityExhausted
                | Kind::CandidateIdentityExhausted
                | Kind::IndexMaintenanceBudgetExceeded
                | Kind::IndexGenerationIdentityExhausted
                | Kind::IdempotencyReceiptNotRetained { .. }
                | Kind::IdempotencyIntentUnverifiable
                | Kind::MutationBindingMismatch
                | Kind::MutationInputMismatch
                | Kind::ElevationTransitionRequired
                | Kind::ElevationRequestProgramMismatch
                | Kind::ElevationApprovalProgramMismatch
                | Kind::ElevationCloseProgramMismatch
                | Kind::MandatoryReviewProgramMismatch
                | Kind::DelegationActivationRequired
                | Kind::CapabilityRevocationRequired
                | Kind::ApplicationProgramRequired
                | Kind::WorkflowAuthorityRequired
                | Kind::ProgramNotActiveOnOccurrence { .. }
                | Kind::ProgramSupportRetired
                | Kind::ProgramActivationUnresolved
                | Kind::RecoveryHandoffMismatch { .. } => failed(identity, commit_denial),
            };
            Err(failure
                .with_commit_denial_kind(kind)
                .with_commit_execution_denial(execution))
        }
        outcome @ (WorthQueryApplicationCommitOutcome::Deferred(_)
        | WorthQueryApplicationCommitOutcome::NoEffect(_)
        | WorthQueryApplicationCommitOutcome::ProductUnpublished(_)
        | WorthQueryApplicationCommitOutcome::Aborted
        | WorthQueryApplicationCommitOutcome::SettlementDeferred(_)
        | WorthQueryApplicationCommitOutcome::Indeterminate(_)) => Err(failed(identity, outcome)),
    }
}

/// The budget a publication companion had no room in when it refused the
/// commit: the request's work, or what the index and its preparation retain.
fn refused_budget(
    outcome: &WorthQueryApplicationCommitOutcome,
) -> Option<WorthQueryOutputDemandDenialKind> {
    use graph::WorthQueryApplicationCommitDeferredKind as Deferred;
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
        WorthQueryApplicationCommitOutcome::Committed(_)
        | WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
        | WorthQueryApplicationCommitOutcome::ProductStale(_)
        | WorthQueryApplicationCommitOutcome::ProductUnpublished(_)
        | WorthQueryApplicationCommitOutcome::Stale(_)
        | WorthQueryApplicationCommitOutcome::Cancelled
        | WorthQueryApplicationCommitOutcome::TimedOut
        | WorthQueryApplicationCommitOutcome::Denied(_)
        | WorthQueryApplicationCommitOutcome::Aborted
        | WorthQueryApplicationCommitOutcome::SettlementDeferred(_)
        | WorthQueryApplicationCommitOutcome::Indeterminate(_) => return None,
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
mod tests;
