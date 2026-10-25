//! Provider compare refusals retain their cause at the application boundary.
use super::provider_execution::WorthQueryProviderProgressionOutcome as Progression;
use super::{
    WorthQueryApplicationCommitDenial as Denial, WorthQueryApplicationCommitDenialStage as Stage,
};
use crate::domain_computation::WorthQueryProviderCompareAndCommitDenial as Compare;

pub(in crate::domain_computation::primary_graph) fn provider_compare_denied(
    denial: Compare,
) -> Progression {
    let denial = match denial {
        Compare::ProviderSession(failure) => {
            if let Some(allocation) = failure.allocation_denial() {
                use worth_execution::{
                    ExecutionAllocationDenialClass as Class, MapKernelStop as Stop,
                    WorkCeilingDenial as Request,
                };
                match allocation.classify() {
                    Class::Request(Request::Stopped(Stop::Cancelled)) => {
                        return Progression::Cancelled
                    }
                    Class::Request(Request::Stopped(Stop::DeadlineElapsed)) => {
                        return Progression::TimedOut
                    }
                    Class::Request(Request::Admission(_))
                    | Class::Request(Request::Stopped(
                        Stop::WorkCounterOverflow
                        | Stop::AccountingUnavailable
                        | Stop::WorkCeiling
                        | Stop::NestedStopped,
                    ))
                    | Class::Request(Request::Panicked)
                    | Class::Physical(_) => {}
                }
            }
            provider_session_denied(failure)
        }
        Compare::DecisionReadSet(failure) => {
            Denial::decision_read_set_denied_at(failure, Stage::ProviderCommit)
        }
    };
    Progression::Denied(denial)
}

pub(in crate::domain_computation::primary_graph) fn provider_session_denied(
    failure: crate::domain_computation::WorthQueryProviderSessionFailure,
) -> Denial {
    use crate::domain_computation::WorthQueryProviderSessionDenialKind as Kind;
    if let Some(stop) = failure.checked_read_stop() {
        return Denial::provider_execution_denied(Stage::ProviderCommit, stop, failure.detail())
            .with_provider_session_failure(failure);
    }
    let kind = match super::super::provider::relational_execution_denial::native_preparation_kind(
        &failure,
    ) {
        Ok(kind) => kind.unwrap_or(failure.kind()),
        Err(stop) => {
            return Denial::provider_execution_denied(
                Stage::ProviderCommit,
                Err(stop),
                failure.detail(),
            )
            .with_provider_session_failure(failure)
        }
    };
    let detail = failure.detail();
    let denial = match kind {
        Kind::IndexMaintenanceBudgetExceeded => {
            Denial::index_maintenance_budget_exceeded(Stage::ProviderCommit, detail)
        }
        Kind::IndexGenerationIdentityExhausted => {
            Denial::index_generation_identity_exhausted(Stage::ProviderCommit)
        }
        Kind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => Denial::active_snapshot_capacity_exhausted(
            Stage::ProviderCommit,
            maximum_active_snapshots,
        ),
        Kind::RetentionCapacityExhausted => {
            Denial::retention_capacity_exhausted(Stage::ProviderCommit)
        }
        Kind::RetentionIdentityExhausted => {
            Denial::retention_identity_exhausted(Stage::ProviderCommit)
        }
        Kind::SnapshotIdentityExhausted => {
            Denial::snapshot_identity_exhausted(Stage::ProviderCommit)
        }
        Kind::CandidateIdentityExhausted => {
            Denial::candidate_identity_exhausted(Stage::ProviderCommit)
        }
        Kind::ExecutionResource { .. }
        | Kind::ExecutionNestedPatternStopped { .. }
        | Kind::ExecutionWorkerPanicked { .. }
        | Kind::ExecutionUncheckedCustomKernel { .. }
        | Kind::ExecutionIdentitiesNotCanonical { .. } => {
            Denial::provider_execution_denied(Stage::ProviderCommit, Ok(kind), detail)
        }
        Kind::ForeignOperationAttempt => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::ForeignExecutionBasis => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::ForeignGraphAuthority => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::UndeclaredOperationScope => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::ResourceEnvelopeMismatch => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::ProviderIdentityMismatch => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::ProviderGenerationMismatch => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::SessionProtocolUnsupported => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::AllocationDenied | Kind::ProviderRejected => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::ProviderPanicked => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::TokenNotMintedForPlan => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::EmptyPhysicalSessionIdentity => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::SessionIdentityExhausted => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
    };
    denial.with_provider_session_failure(failure)
}

pub(in crate::domain_computation::primary_graph::application_attempt) fn read_set_failure(
    failure: crate::domain_computation::WorthQueryDecisionReadSetFailure,
) -> Progression {
    use crate::domain_computation::WorthQueryDecisionReadSetDenialKind as Kind;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Stop;
    match failure.kind() {
        Kind::RequestInterrupted(Stop::Cancelled) => Progression::Cancelled,
        Kind::RequestInterrupted(Stop::DeadlineExceeded) => Progression::TimedOut,
        _ => Progression::Denied(Denial::decision_read_set_denied(failure)),
    }
}

#[cfg(test)]
mod tests;

pub(in crate::domain_computation::primary_graph) fn provider_session_execution_denied(
    stop: Result<
        crate::domain_computation::WorthQueryProviderSessionDenialKind,
        crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    >,
    stage: Stage,
    detail: impl Into<std::sync::Arc<str>>,
) -> Denial {
    // The preparation refusal retains its category and retry policy while
    // carrying the distinct cause and the stage that stopped it.
    Denial::provider_execution_denied(stage, stop, detail)
}

pub(in crate::domain_computation::primary_graph) fn control_stopped_outcome(
    stopped: crate::domain_computation::WorthQueryProviderSessionCommitControlStopped,
) -> Progression {
    if stopped.is_execution_preparation() {
        // A preparation stop is pre-effect and remains retryable without
        // entering post-effect recovery.
        return Progression::Denied(Denial::provider_execution_denied(
            Stage::ProviderCommit,
            Err(stopped.kind()),
            stopped.detail(),
        ));
    }
    match stopped.kind() {
        crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled => {
            Progression::Cancelled
        }
        crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut => {
            Progression::TimedOut
        }
    }
}
