use super::super::{
    WorthQueryTemporalReentryOutcome as Outcome, WorthQueryTemporalTerminalFailure as Terminal,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenial as Denial, WorthQueryApplicationCommitDenialKind as Kind,
    WorthQueryApplicationCommitDenialStage as Stage,
    WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel as Level,
};
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Session,
};

pub(super) fn classify_denial(denial: &Denial) -> Outcome {
    if let Some(cause) = denial.execution_denial_cause() {
        return match (denial.stage(), cause) {
            (Stage::ProviderCommit, Err(Control::Cancelled)) => {
                Outcome::RetryableExecutionControlStopped(Control::Cancelled)
            }
            (Stage::ProviderCommit, Err(Control::TimedOut)) => {
                Outcome::RetryableExecutionControlStopped(Control::TimedOut)
            }
            (
                stage @ (Stage::ProposalBinding
                | Stage::BridgePlanning
                | Stage::BasisAdmission
                | Stage::ResourceAdmission
                | Stage::ManagedRunAdmission
                | Stage::ProviderPlan
                | Stage::Idempotency
                | Stage::DecisionReadSet
                | Stage::EffectLowering
                | Stage::ElevationTransition
                | Stage::DelegationTransition
                | Stage::ProvisionalState
                | Stage::InvariantExecution),
                Err(kind @ (Control::Cancelled | Control::TimedOut)),
            ) => Outcome::TerminalFailure(Terminal::ApplicationExecution {
                stage,
                cause: Err(kind),
            }),
            (
                stage @ (Stage::ProposalBinding
                | Stage::BridgePlanning
                | Stage::BasisAdmission
                | Stage::ResourceAdmission
                | Stage::ManagedRunAdmission
                | Stage::ProviderPlan
                | Stage::Idempotency
                | Stage::DecisionReadSet
                | Stage::EffectLowering
                | Stage::ElevationTransition
                | Stage::DelegationTransition
                | Stage::ProvisionalState
                | Stage::InvariantExecution
                | Stage::ProviderCommit),
                Ok(
                    kind @ (Session::ExecutionResource {
                        denial:
                            Resource::MemoryLimit {
                                level: Level::Policy | Level::Process | Level::Declared,
                                ..
                            }
                            | Resource::WorkExhausted
                            | Resource::WorkCounterOverflow
                            | Resource::RetainedBytesExhausted
                            | Resource::ScratchCapacityExceeded
                            | Resource::ResultCapacityExceeded
                            | Resource::CapacityOverflow
                            | Resource::ChargedBytesOverflow
                            | Resource::WorkerLimit
                            | Resource::PolicyMemoryLimit
                            | Resource::WorkLimit
                            | Resource::NestedLeaseMisuse
                            | Resource::NestedAdvancementOpening
                            | Resource::ForeignAdvancementPhase
                            | Resource::NoActiveExecutionScope
                            | Resource::EquivalenceContractUnavailable,
                        ..
                    }
                    | Session::ForeignOperationAttempt
                    | Session::ForeignExecutionBasis
                    | Session::ForeignGraphAuthority
                    | Session::UndeclaredOperationScope
                    | Session::ResourceEnvelopeMismatch
                    | Session::ExecutionNestedPatternStopped { .. }
                    | Session::ExecutionWorkerPanicked { .. }
                    | Session::ExecutionIdentitiesNotCanonical { .. }
                    | Session::ExecutionUncheckedCustomKernel { .. }
                    | Session::ActiveSnapshotCapacityExhausted { .. }
                    | Session::RetentionCapacityExhausted
                    | Session::RetentionIdentityExhausted
                    | Session::SnapshotIdentityExhausted
                    | Session::CandidateIdentityExhausted
                    | Session::PreparedRootBudgetExhausted { .. }
                    | Session::IndexMaintenanceBudgetExceeded
                    | Session::IndexGenerationIdentityExhausted
                    | Session::ProviderIdentityMismatch
                    | Session::ProviderGenerationMismatch
                    | Session::SessionProtocolUnsupported
                    | Session::ProviderRejected
                    | Session::ProviderPanicked
                    | Session::TokenNotMintedForPlan
                    | Session::EmptyPhysicalSessionIdentity
                    | Session::SessionIdentityExhausted),
                ),
            ) => Outcome::TerminalFailure(Terminal::ApplicationExecution {
                stage,
                cause: Ok(kind),
            }),
        };
    }
    classify_kind(denial.kind())
}

fn classify_kind(kind: Kind) -> Outcome {
    match kind {
        Kind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => Outcome::SnapshotCapacityBackpressured {
            maximum_active_snapshots,
        },
        Kind::RetentionCapacityExhausted => Outcome::RetentionCapacityBackpressured,
        // All of these could reach HEAD's Execution -> Aborted -> retry route.
        Kind::ExecutionWorkerPanicked { .. }
        | Kind::ExecutionNestedPatternStopped { .. }
        | Kind::ExecutionUncheckedCustomKernel { .. }
        | Kind::ExecutionIdentitiesNotCanonical { .. } => Outcome::RetryableCommitFailure(kind),
        Kind::ExecutionResource { denial, .. } => match denial {
            Resource::MemoryLimit {
                level: Level::Process | Level::Policy | Level::Declared,
                ..
            }
            | Resource::WorkExhausted
            | Resource::ResultCapacityExceeded
            | Resource::WorkCounterOverflow
            | Resource::ChargedBytesOverflow
            | Resource::CapacityOverflow
            | Resource::NestedLeaseMisuse
            | Resource::NoActiveExecutionScope
            | Resource::ScratchCapacityExceeded => Outcome::RetryableCommitFailure(kind),
            // Opening another request on the same thread is a caller defect.
            Resource::NestedAdvancementOpening => {
                Outcome::TerminalFailure(Terminal::ApplicationCommit(kind))
            }
            // A phase from another installed runtime is a caller defect, never pressure.
            Resource::ForeignAdvancementPhase => {
                Outcome::TerminalFailure(Terminal::ApplicationCommit(kind))
            }
            // Unreachable at HEAD: Relational's remaining-work child retains the
            // parent's workers, memory and determinism and lowers its work limit.
            Resource::WorkerLimit
            | Resource::PolicyMemoryLimit
            | Resource::WorkLimit
            | Resource::EquivalenceContractUnavailable => {
                Outcome::TerminalFailure(Terminal::ApplicationCommit(kind))
            }
            // Retained state is not a Relational execution cause; scratch is.
            Resource::RetainedBytesExhausted => {
                Outcome::TerminalFailure(Terminal::ApplicationCommit(kind))
            }
        },
        Kind::ProviderRejected
        | Kind::CustomInvariantDenied
        | Kind::CandidateValidatorWorkExceeded { .. }
        | Kind::WorkflowSettlementDenied { .. }
        | Kind::ProductBasisStale
        | Kind::UniqueValueTaken
        | Kind::UniqueIndexUnavailable
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
        | Kind::ProgramActivationUnresolved
        | Kind::RecoveryHandoffMismatch { .. } => {
            Outcome::TerminalFailure(Terminal::ApplicationCommit(kind))
        }
    }
}

#[cfg(test)]
pub(super) mod tests;
