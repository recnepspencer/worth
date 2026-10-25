//! A failed managed cleanup retains the unresolved commit posture.
use crate::domain_computation::primary_graph::application_attempt::provider_execution::{
    outcome::WorthQueryProviderProgressionOutcome as Progression,
    recovery_evidence::unknown_commit_recovery_evidence,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind as Kind, WorthQueryApplicationUncommitted as Outcome,
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
            Kind::StorageUnavailable(failure) => {
                cleanup_execution_indeterminate(Ok(Session::StorageUnavailable(failure)))
            }
            Kind::ExecutionResource {
                denial,
                partition_identity,
                policy_ancestor,
            } => cleanup_execution_indeterminate(Ok(Session::ExecutionResource {
                denial,
                partition_identity,
                policy_ancestor,
            })),
            Kind::ExecutionNestedPatternStopped { partition_identity } => {
                cleanup_execution_indeterminate(Ok(Session::ExecutionNestedPatternStopped {
                    partition_identity,
                }))
            }
            Kind::ExecutionWorkerPanicked { partition_identity } => {
                cleanup_execution_indeterminate(Ok(Session::ExecutionWorkerPanicked {
                    partition_identity,
                }))
            }
            Kind::ExecutionUncheckedCustomKernel { partition_identity } => {
                cleanup_execution_indeterminate(Ok(Session::ExecutionUncheckedCustomKernel {
                    partition_identity,
                }))
            }
            Kind::ExecutionIdentitiesNotCanonical { partition_identity } => {
                cleanup_execution_indeterminate(Ok(Session::ExecutionIdentitiesNotCanonical {
                    partition_identity,
                }))
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
            | Kind::IndexMaintenanceBudgetExceeded
            | Kind::IndexGenerationIdentityExhausted
            | Kind::IdempotencyIntentDrift
            | Kind::IdempotencyReceiptNotRetained { .. }
            | Kind::IdempotencyIntentUnverifiable
            | Kind::MutationBindingMismatch
            | Kind::MutationInputMismatch
            | Kind::OperationBodyExecutionRequired
            | Kind::SignedCreationAuthentication { .. }
            | Kind::SignedEntityImmutable
            | Kind::SigningLifecycleForgery
            | Kind::SigningLifecycleUnavailable
            | Kind::SigningLifecycleEvidenceCapacity { .. }
            | Kind::SigningLifecycleSubjectCapacity { .. }
            | Kind::SigningLifecycleRepresentationCapacity
            | Kind::SigningLifecycleObservation { .. }
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
            | Kind::CorrectionContextMismatch
            | Kind::CorrectionIdentityMismatch => cleanup_indeterminate(),
        },
        Progression::Indeterminate(evidence) => {
            Outcome::Indeterminate(evidence.with_cleanup_failure())
        }
        Progression::Cancelled => cleanup_execution_indeterminate(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
        )),
        Progression::TimedOut => cleanup_execution_indeterminate(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
        )),
        Progression::Committed(_)
        | Progression::AlreadyCommitted(_)
        | Progression::Stale(_)
        | Progression::Aborted
        | Progression::Deferred(_) => cleanup_indeterminate(),
    }
}

// The existing unresolved evidence already holds a provider-session kind.
// A cleanup failure still requires recovery; its cause cannot make it retryable.
fn cleanup_execution_indeterminate(
    kind: Result<Session, crate::domain_computation::WorthQueryProviderSessionControlStopKind>,
) -> Outcome {
    use crate::domain_computation::primary_graph::{
        WorthQueryApplicationCommitRecoveryKind as Recovery,
        WorthQueryApplicationUnresolvedCommitEvidence as Evidence,
    };
    let failure = WorthQueryProviderSessionFailure::checked_read_stopped(
        kind,
        WorthQueryProviderSessionProtocolStage::Commit,
        "managed mutation run failed to finish after provider progression",
    );
    Outcome::Indeterminate(
        Evidence::from_provider_session_failure(Recovery::CommitRecoveryRequired, &failure)
            .with_cleanup_failure(),
    )
}

fn cleanup_indeterminate() -> Outcome {
    Outcome::Indeterminate(
        unknown_commit_recovery_evidence(
            "managed mutation run failed to finish after provider progression",
        )
        .with_cleanup_failure(),
    )
}

#[cfg(test)]
mod tests;
