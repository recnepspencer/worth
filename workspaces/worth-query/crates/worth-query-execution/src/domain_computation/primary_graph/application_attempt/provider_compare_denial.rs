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
                use worth_execution::ExecutionAllocationDenialKind as Kind;
                match allocation.kind() {
                    Kind::Cancelled => return Progression::Cancelled,
                    Kind::DeadlineElapsed => return Progression::TimedOut,
                    Kind::Layout
                    | Kind::Lease(_)
                    | Kind::Allocator
                    | Kind::CapacityMismatch
                    | Kind::WriteBeyondReserved
                    | Kind::IncompleteSeal => {}
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
    let kind =
        super::super::provider::relational_execution_denial::native_preparation_kind(&failure)
            .unwrap_or(failure.kind());
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

#[cfg(test)]
pub(in crate::domain_computation::primary_graph) mod preparation_outcome;
#[cfg(test)]
mod tests;

pub(in crate::domain_computation::primary_graph) fn provider_session_kind_denied(
    kind: crate::domain_computation::WorthQueryProviderSessionDenialKind,
    stage: Stage,
    detail: impl Into<std::sync::Arc<str>>,
) -> Denial {
    // The preparation refusal retains its category and retry policy while
    // carrying the distinct cause and the stage that stopped it.
    Denial::provider_execution_denied(stage, Ok(kind), detail)
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

#[cfg(test)]
pub(in crate::domain_computation::primary_graph) fn provider_session_control_denied(
    kind: crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    stage: Stage,
    detail: impl Into<std::sync::Arc<str>>,
) -> Denial {
    Denial::provider_execution_denied(stage, Err(kind), detail)
}
