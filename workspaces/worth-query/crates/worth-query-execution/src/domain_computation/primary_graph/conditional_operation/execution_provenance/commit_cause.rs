use super::WorthQueryConditionalExecutionCause;
pub(super) fn application_commit_cause(
    kind: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind,
) -> WorthQueryConditionalExecutionCause {
    use crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind as Kind;
    match kind {
        Kind::IdempotencyPreparationBusy
        | Kind::ContentInconsistency
        | Kind::Content(_)
        | Kind::StorageUnavailable(_)
        | Kind::ExecutionResource { .. }
        | Kind::ExecutionNestedPatternStopped { .. }
        | Kind::ExecutionWorkerPanicked { .. }
        | Kind::ExecutionUncheckedCustomKernel { .. }
        | Kind::ExecutionIdentitiesNotCanonical { .. } => {
            WorthQueryConditionalExecutionCause::ApplicationCommitDenied(kind)
        }
        Kind::UnobservedWrite
        | Kind::ProviderRejected
        | Kind::CustomInvariantDenied
        | Kind::DomainValueRejected
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
        | Kind::IdempotencyForeignAffinity
        | Kind::IdempotencyIntentDrift
        | Kind::OriginalEffectReleased
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
        | Kind::StaleIncarnation
        | Kind::CorrectionContextMismatch
        | Kind::CorrectionIdentityMismatch => WorthQueryConditionalExecutionCause::TerminalFailure,
    }
}
