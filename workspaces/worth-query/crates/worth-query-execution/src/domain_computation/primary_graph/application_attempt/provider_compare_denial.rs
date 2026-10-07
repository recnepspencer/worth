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
        Compare::ProviderSession(failure) => provider_session_denied(failure),
        Compare::DecisionReadSet(failure) => decision_read_set_denied(failure),
    };
    Progression::Denied(denial)
}

pub(in crate::domain_computation::primary_graph) fn provider_session_denied(
    failure: crate::domain_computation::WorthQueryProviderSessionFailure,
) -> Denial {
    use crate::domain_computation::WorthQueryProviderSessionDenialKind as Kind;
    let detail = failure.detail();
    match failure.kind() {
        Kind::IndexMaintenanceBudgetExceeded => {
            Denial::index_maintenance_budget_exceeded(Stage::ProviderCommit)
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
        Kind::PreparedRootBudgetExhausted {
            maximum_bytes,
            required_bytes,
        } => Denial::prepared_root_budget_exhausted(
            Stage::ProviderCommit,
            maximum_bytes,
            required_bytes,
        ),
        Kind::ExecutionResource {
            denial,
            partition_identity,
            policy_ancestor,
        } => Denial::execution_resource(denial, partition_identity, policy_ancestor, detail),
        Kind::ExecutionNestedPatternStopped { partition_identity } => {
            Denial::execution_nested_stopped(partition_identity, detail)
        }
        Kind::ExecutionWorkerPanicked { partition_identity } => {
            Denial::execution_worker_panicked(partition_identity, detail)
        }
        Kind::ExecutionUncheckedCustomKernel { partition_identity } => {
            Denial::execution_unchecked_custom_kernel(partition_identity, detail)
        }
        Kind::ExecutionIdentitiesNotCanonical { partition_identity } => {
            Denial::execution_identities_not_canonical(partition_identity, detail)
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
        Kind::ProviderRejected => {
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
    }
}

fn decision_read_set_denied(
    failure: crate::domain_computation::WorthQueryDecisionReadSetFailure,
) -> Denial {
    use crate::domain_computation::WorthQueryDecisionReadSetDenialKind as Kind;
    let detail = failure.detail();
    match failure.kind() {
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
        Kind::InvalidRequest => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::UndeclaredFamily => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::FamilyKindMismatch => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::DecisionFactsUnsupported => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::InvalidProviderEvidence => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::EvidenceSubstitution => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::IncompleteRequiredFamilies => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::IncompleteRequiredFacts => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::DecisionFactBudgetExceeded => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::ProviderRejected => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
        Kind::ProviderPanicked => {
            Denial::provider_rejected_with_detail(Stage::ProviderCommit, detail)
        }
    }
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
    // These owners reported ProviderRejected at their own stage at HEAD.
    // Distinguishing its evidence must not change that category or retry policy.
    Denial::provider_execution_denied(stage, Ok(kind), detail)
}

pub(in crate::domain_computation::primary_graph) fn control_stopped_outcome(
    stopped: crate::domain_computation::WorthQueryProviderSessionCommitControlStopped,
) -> Progression {
    if stopped.is_execution_preparation() {
        // HEAD recovered every preparation Execution refusal as Aborted.
        // Preserve its retry policy without requesting post-effect recovery.
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

pub(in crate::domain_computation::primary_graph) fn provider_session_control_denied(
    kind: crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    stage: Stage,
    detail: impl Into<std::sync::Arc<str>>,
) -> Denial {
    Denial::provider_execution_denied(stage, Err(kind), detail)
}
