use super::WorthQueryConditionalExecutionCause;
pub(super) fn application_commit_cause(
    kind: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind,
) -> WorthQueryConditionalExecutionCause {
    use crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind as Kind;
    match kind {
        Kind::ExecutionResource { .. }
        | Kind::ExecutionNestedPatternStopped { .. }
        | Kind::ExecutionWorkerPanicked { .. }
        | Kind::ExecutionUncheckedCustomKernel { .. }
        | Kind::ExecutionIdentitiesNotCanonical { .. } => {
            WorthQueryConditionalExecutionCause::ApplicationCommitDenied(kind)
        }
        Kind::ProviderRejected
        | Kind::CustomInvariantDenied
        | Kind::CandidateValidatorWorkExceeded { .. }
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
        | Kind::IdempotencyWindowExpired
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
        | Kind::ProgramActivationUnresolved => WorthQueryConditionalExecutionCause::TerminalFailure,
    }
}
