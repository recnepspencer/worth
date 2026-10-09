//! A failed managed cleanup retains the unresolved commit posture.
use crate::domain_computation::primary_graph::application_attempt::provider_execution::{
    outcome::WorthQueryProviderProgressionOutcome as Progression,
    recovery_evidence::unknown_commit_recovery_evidence,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind as Kind, WorthQueryApplicationCommitOutcome as Outcome,
};
use crate::domain_computation::{
    WorthQueryProviderSessionDenialKind as Session, WorthQueryProviderSessionFailure,
    WorthQueryProviderSessionProtocolStage,
};

pub(super) fn cleanup_failed_outcome(outcome: Progression) -> Outcome {
    match outcome {
        Progression::ProductUnpublished(unpublished) => Outcome::ProductUnpublished(unpublished),
        Progression::ProductStale(stale) => Outcome::ProductStale(stale),
        Progression::NoEffect(no_effect) => Outcome::NoEffect(
            crate::domain_computation::primary_graph::WorthQueryApplicationNoEffect::from_world(
                no_effect,
            ),
        ),
        Progression::Denied(denial) => match denial.kind() {
            Kind::ExecutionResource {
                denial,
                partition_identity,
                policy_ancestor,
            } => cleanup_execution_indeterminate(Session::ExecutionResource {
                denial,
                partition_identity,
                policy_ancestor,
            }),
            Kind::ExecutionNestedPatternStopped { partition_identity } => {
                cleanup_execution_indeterminate(Session::ExecutionNestedPatternStopped {
                    partition_identity,
                })
            }
            Kind::ExecutionWorkerPanicked { partition_identity } => {
                cleanup_execution_indeterminate(Session::ExecutionWorkerPanicked {
                    partition_identity,
                })
            }
            Kind::ExecutionUncheckedCustomKernel { partition_identity } => {
                cleanup_execution_indeterminate(Session::ExecutionUncheckedCustomKernel {
                    partition_identity,
                })
            }
            Kind::ExecutionIdentitiesNotCanonical { partition_identity } => {
                cleanup_execution_indeterminate(Session::ExecutionIdentitiesNotCanonical {
                    partition_identity,
                })
            }
            Kind::ProviderRejected
            | Kind::CustomInvariantDenied
            | Kind::WorkflowSettlementDenied { .. }
            | Kind::ProductBasisStale
            | Kind::UniqueValueTaken
            | Kind::UniqueIndexUnavailable
            | Kind::ActiveSnapshotCapacityExhausted { .. }
            | Kind::RetentionCapacityExhausted
            | Kind::RetentionIdentityExhausted
            | Kind::SnapshotIdentityExhausted
            | Kind::CandidateIdentityExhausted
            | Kind::PreparedRootBudgetExhausted { .. }
            | Kind::IndexMaintenanceBudgetExceeded
            | Kind::IndexGenerationIdentityExhausted
            | Kind::IdempotencyIntentDrift
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
            | Kind::RecoveryHandoffMismatch { .. } => cleanup_indeterminate(),
        },
        Progression::Committed(_)
        | Progression::AlreadyCommitted(_)
        | Progression::Stale(_)
        | Progression::Cancelled
        | Progression::TimedOut
        | Progression::Aborted
        | Progression::Deferred(_)
        | Progression::SettlementDeferred(_)
        | Progression::Indeterminate(_) => cleanup_indeterminate(),
    }
}

// The existing unresolved evidence already holds a provider-session kind.
// A cleanup failure still requires recovery; its cause cannot make it retryable.
fn cleanup_execution_indeterminate(kind: Session) -> Outcome {
    use crate::domain_computation::primary_graph::{
        WorthQueryApplicationCommitRecoveryKind as Recovery,
        WorthQueryApplicationUnresolvedCommitEvidence as Evidence,
    };
    let failure = WorthQueryProviderSessionFailure::new(
        kind,
        WorthQueryProviderSessionProtocolStage::Commit,
        "managed mutation run failed to finish after provider progression",
        Default::default(),
    );
    Outcome::Indeterminate(Evidence::from_provider_session_failure(
        Recovery::CommitRecoveryRequired,
        &failure,
    ))
}

fn cleanup_indeterminate() -> Outcome {
    Outcome::Indeterminate(unknown_commit_recovery_evidence(
        "managed mutation run failed to finish after provider progression",
    ))
}

#[cfg(test)]
mod tests;
