//! A commit refusal retains its native cause and publication staleness.
use super::*;
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Native,
};
use graph::VerificationCapacityStop;
use graph::WorthQueryApplicationCommitDenialStage as Stage;

pub(super) fn stopped(
    identity: &str,
    kind: WorthQueryApplicationCommitDenialKind,
    stage: Stage,
    cause: Option<Result<Native, Control>>,
    capacity: Option<VerificationCapacityStop>,
    detail: &impl std::fmt::Debug,
) -> WorthQueryOutputDemandDenial {
    use WorthQueryApplicationCommitDenialKind as Kind;
    let execution = cause.map(|cause| (stage, cause));
    let failure = match (execution, kind) {
        (_, Kind::ProductBasisStale) => denial(
            WorthQueryOutputDemandDenialKind::PublicationStale,
            identity.to_owned(),
        )
        .with_publication_stale(Some(ProducerPublicationStale::CommitProductBasisStale {
            stage,
        })),
        (Some((_, cause)), _) => commit_execution_stopped(identity, detail, cause),
        (None, Kind::IdempotencyIntentDrift) => denial(
            WorthQueryOutputDemandDenialKind::ProducerUnavailable,
            format!("{identity}: {detail:?}"),
        ),
        (
            None,
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
            | Kind::OperationBodyExecutionRequired
            | Kind::SignedCreationAuthentication { .. }
            | Kind::SignedEntityImmutable
            | Kind::SigningLifecycleForgery
            | Kind::SigningLifecycleUnavailable
            | Kind::SigningLifecycleEvidenceCapacity { .. }
            | Kind::SigningLifecycleSubjectCapacity { .. }
            | Kind::SigningLifecycleRepresentationCapacity
            | Kind::SigningLifecycleObservation { .. }
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
            | Kind::CorrectionContextMismatch
            | Kind::CorrectionIdentityMismatch,
        ) => failed(identity, detail),
    };
    failure
        .with_commit_denial_kind(kind)
        .with_commit_execution_denial(execution)
        .with_capacity_stop(capacity)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_product_basis_wins_over_a_carried_execution_stop() {
        let stop = stopped(
            "producer",
            WorthQueryApplicationCommitDenialKind::ProductBasisStale,
            Stage::InvariantExecution,
            Some(Err(Control::Cancelled)),
            None,
            &"stale plus cancellation",
        );
        assert_eq!(
            stop.kind(),
            WorthQueryOutputDemandDenialKind::PublicationStale
        );
        assert_eq!(
            stop.commit_execution_denial(),
            Some((Stage::InvariantExecution, Err(Control::Cancelled)))
        );
        let expected = WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::PublicationStale,
            "producer",
        )
        .with_publication_stale(Some(ProducerPublicationStale::CommitProductBasisStale {
            stage: Stage::InvariantExecution,
        }))
        .with_commit_denial_kind(WorthQueryApplicationCommitDenialKind::ProductBasisStale)
        .with_commit_execution_denial(Some((Stage::InvariantExecution, Err(Control::Cancelled))));
        assert_eq!(
            stop, expected,
            "staleness and original execution evidence both survive"
        );
    }
}
