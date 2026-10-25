//! Bank-owned denials raised before an application commit begins.

use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use worth_query_host::facade::runtime::ExecutionAllocationDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankApplicationAttemptDenialKind {
    StorageUnavailable(worth_query_host::facade::primary_graph::RelationalStorageUnavailable),
    ValueDecode(worth_query_host::facade::primary_graph::WorthQueryValueDecodeDenial),
    ForeignPhase,
    ExecutionRequest(worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind),
    Cancelled,
    DeadlineExceeded,
    ForeignApplication,
    ProjectionAdmissionMismatch,
    CurrentAuthorityDenied,
    OutsideRealizedReadScope,
    UndeclaredDecisionRead,
    StaleEntityIdentity,
    MissingAuthoritativeFact,
    InvalidAuthoritativeValue,
    IncompleteDecisionReadSet,
    DecisionDependencyMismatch,
    DecisionFactBudgetExceeded,
    AllocationDenied,
    RetainedSourceStorageDenied,
    CorrectionCaptureDenied,
    MutationPreconditionMismatch,
    SourceRetired,
    SourceChanged,
    AmbiguousRelation,
    UndeclaredEffect,
    InvalidEffectValue,
    ForeignEffectTarget,
    InvalidOutputRole,
    ForeignOutputRole,
    UndeclaredOutputRole,
    MissingOutputRole,
    DuplicateOutputRole,
    OutputRoleEntityMismatch,
    OutputRoleActionMismatch,
    OutputRoleCardinalityMismatch,
    DuplicateEffectKey,
    CandidateCapacityExceeded,
    ExternalEffectPayloadProjectionRejected,
    ForeignConditionalDefinitionChange,
    DuplicateConditionalDefinitionChange,
    IncompleteEffectBasis,
    UniqueValueTaken,
    UniqueIndexUnavailable,
    DelegationActivationRequired,
    DelegationActivationProgramMismatch,
    CapabilityRevocationRequired,
    CapabilityRevocationProgramMismatch,
    ElevationTransitionRequired,
    ElevationRequestProgramMismatch,
    ElevationApprovalProgramMismatch,
    ElevationCloseProgramMismatch,
    MandatoryReviewProgramMismatch,
    WorkflowDefinitionAffinityMismatch,
    WorkflowDefinitionAuthorityMismatch,
    WorkflowDefinitionIntentIdentityUnavailable,
    WorkflowLineageUnavailable,
    WorkflowDefinitionCompilationUnavailable,
    WorkflowDefinitionMissingOperationInput,
    CapturedOperationInputsNotSupported,
    WorkflowDefinitionSuperseded,
    WorkflowDefinitionRetired,
    WorkflowInstanceAffinityMismatch,
    WorkflowInstanceAuthorityMismatch,
    WorkflowInstanceNotCompleted,
    WorkflowInstanceAlreadySucceeded,
    WorkflowPredecessorOutputUnavailable,
    WorkflowPredecessorOutputMismatch,
    WorkflowInstanceIntentIdentityUnavailable,
    WorkflowInstanceOriginalIntentNotFound,
    WorkflowInstanceOriginalIntentAmbiguous,
    WorkflowInstanceOriginalIntentUnavailable,
    WorkflowInstanceCapacityUnavailable,
    WorkflowLineageCapacityUnavailable,
    WorkflowInstanceCancelled,
    WorkflowInstanceTerminated,
    WorkflowInstanceCompleted,
    WorkflowOperationInOwnerCustody,
    WorkflowInstanceHistoryUnavailable,
    WorkflowInstanceMigrated,
    WorkflowInstanceMigrationUnmapped,
    WorkflowInstanceDeadlineElapsed,
    WorkflowTrustedTimeUnavailable,
    WorkflowInstanceEvidenceCapacityUnavailable,
    WorkflowTransitionAffinityMismatch,
    WorkflowTransitionAuthorityMismatch,
    WorkflowTransitionAlreadySettled,
    WorkflowTransitionNodeUnsupported,
    WorkflowTransitionOperationUnsettled,
    WorkflowTransitionIdentityUnavailable,
    WorkflowConditionExpressionDenied,
    OperationBodyRequirementExpressionDenied,
    OperationBodyValueExpressionDenied,
    WorkflowAssessmentEvidenceIncomplete,
    WorkflowAssessmentEvidenceMismatch,
    WorkflowApprovalPrincipalStale,
    WorkflowApprovalGrantUnavailable,
    WorkflowApprovalExpired,
    WorkflowApprovalDelegationChanged,
    WorkflowApprovalAuthorityDenied,
    ConflictingEffectStep,
    WorkflowHistoryReconstructionBudgetExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BankCommitPreparationDenial {
    Application {
        kind: BankApplicationAttemptDenialKind,
        /// Physical allocation refusal, including its original checked quote.
        allocation: Option<ExecutionAllocationDenial>,
    },
    InvalidProposalShape,
    AccountingRevisionOverflow,
}

impl From<WorthQueryApplicationAttemptDenial> for BankCommitPreparationDenial {
    fn from(denial: WorthQueryApplicationAttemptDenial) -> Self {
        Self::Application {
            kind: application_attempt_kind(denial.kind()),
            allocation: denial.allocation_denial().cloned(),
        }
    }
}

impl std::fmt::Display for BankCommitPreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "bank commit preparation denied: {self:?}")
    }
}

impl std::error::Error for BankCommitPreparationDenial {}

const fn application_attempt_kind(
    kind: WorthQueryApplicationAttemptDenialKind,
) -> BankApplicationAttemptDenialKind {
    use WorthQueryApplicationAttemptDenialKind as Query;
    match kind {
        Query::StorageUnavailable(failure) => {
            BankApplicationAttemptDenialKind::StorageUnavailable(failure)
        }
        Query::ValueDecode(cause) => BankApplicationAttemptDenialKind::ValueDecode(cause),
        Query::ForeignPhase => BankApplicationAttemptDenialKind::ForeignPhase,
        Query::ExecutionRequest(cause) => BankApplicationAttemptDenialKind::ExecutionRequest(cause),
        Query::Cancelled => BankApplicationAttemptDenialKind::Cancelled,
        Query::DeadlineExceeded => BankApplicationAttemptDenialKind::DeadlineExceeded,
        Query::ForeignApplication => BankApplicationAttemptDenialKind::ForeignApplication,
        Query::ProjectionAdmissionMismatch => {
            BankApplicationAttemptDenialKind::ProjectionAdmissionMismatch
        }
        Query::CurrentAuthorityDenied => BankApplicationAttemptDenialKind::CurrentAuthorityDenied,
        Query::CorrectionCaptureDenied => BankApplicationAttemptDenialKind::CorrectionCaptureDenied,
        Query::OutsideRealizedReadScope => {
            BankApplicationAttemptDenialKind::OutsideRealizedReadScope
        }
        Query::UndeclaredDecisionRead => BankApplicationAttemptDenialKind::UndeclaredDecisionRead,
        Query::StaleEntityIdentity => BankApplicationAttemptDenialKind::StaleEntityIdentity,
        Query::MissingAuthoritativeFact => {
            BankApplicationAttemptDenialKind::MissingAuthoritativeFact
        }
        Query::InvalidAuthoritativeValue => {
            BankApplicationAttemptDenialKind::InvalidAuthoritativeValue
        }
        Query::IncompleteDecisionReadSet => {
            BankApplicationAttemptDenialKind::IncompleteDecisionReadSet
        }
        Query::DecisionDependencyMismatch => {
            BankApplicationAttemptDenialKind::DecisionDependencyMismatch
        }
        Query::DecisionFactBudgetExceeded => {
            BankApplicationAttemptDenialKind::DecisionFactBudgetExceeded
        }
        Query::AllocationDenied => BankApplicationAttemptDenialKind::AllocationDenied,
        Query::RetainedSourceStorageDenied => {
            BankApplicationAttemptDenialKind::RetainedSourceStorageDenied
        }
        Query::MutationPreconditionMismatch => {
            BankApplicationAttemptDenialKind::MutationPreconditionMismatch
        }
        Query::SourceRetired => BankApplicationAttemptDenialKind::SourceRetired,
        Query::SourceChanged => BankApplicationAttemptDenialKind::SourceChanged,
        Query::AmbiguousRelation => BankApplicationAttemptDenialKind::AmbiguousRelation,
        Query::UndeclaredEffect => BankApplicationAttemptDenialKind::UndeclaredEffect,
        Query::InvalidEffectValue => BankApplicationAttemptDenialKind::InvalidEffectValue,
        Query::ForeignEffectTarget => BankApplicationAttemptDenialKind::ForeignEffectTarget,
        Query::InvalidOutputRole => BankApplicationAttemptDenialKind::InvalidOutputRole,
        Query::ForeignOutputRole => BankApplicationAttemptDenialKind::ForeignOutputRole,
        Query::UndeclaredOutputRole => BankApplicationAttemptDenialKind::UndeclaredOutputRole,
        Query::MissingOutputRole => BankApplicationAttemptDenialKind::MissingOutputRole,
        Query::DuplicateOutputRole => BankApplicationAttemptDenialKind::DuplicateOutputRole,
        Query::OutputRoleEntityMismatch => {
            BankApplicationAttemptDenialKind::OutputRoleEntityMismatch
        }
        Query::OutputRoleActionMismatch => {
            BankApplicationAttemptDenialKind::OutputRoleActionMismatch
        }
        Query::OutputRoleCardinalityMismatch => {
            BankApplicationAttemptDenialKind::OutputRoleCardinalityMismatch
        }
        Query::DuplicateEffectKey => BankApplicationAttemptDenialKind::DuplicateEffectKey,
        Query::CandidateCapacityExceeded => {
            BankApplicationAttemptDenialKind::CandidateCapacityExceeded
        }
        Query::ExternalEffectPayloadProjectionRejected => {
            BankApplicationAttemptDenialKind::ExternalEffectPayloadProjectionRejected
        }
        Query::ForeignConditionalDefinitionChange => {
            BankApplicationAttemptDenialKind::ForeignConditionalDefinitionChange
        }
        Query::DuplicateConditionalDefinitionChange => {
            BankApplicationAttemptDenialKind::DuplicateConditionalDefinitionChange
        }
        Query::IncompleteEffectBasis => BankApplicationAttemptDenialKind::IncompleteEffectBasis,
        Query::UniqueValueTaken => BankApplicationAttemptDenialKind::UniqueValueTaken,
        Query::UniqueIndexUnavailable => BankApplicationAttemptDenialKind::UniqueIndexUnavailable,
        Query::DelegationActivationRequired => {
            BankApplicationAttemptDenialKind::DelegationActivationRequired
        }
        Query::DelegationActivationProgramMismatch => {
            BankApplicationAttemptDenialKind::DelegationActivationProgramMismatch
        }
        Query::CapabilityRevocationRequired => {
            BankApplicationAttemptDenialKind::CapabilityRevocationRequired
        }
        Query::CapabilityRevocationProgramMismatch => {
            BankApplicationAttemptDenialKind::CapabilityRevocationProgramMismatch
        }
        Query::ElevationTransitionRequired => {
            BankApplicationAttemptDenialKind::ElevationTransitionRequired
        }
        Query::ElevationRequestProgramMismatch => {
            BankApplicationAttemptDenialKind::ElevationRequestProgramMismatch
        }
        Query::ElevationApprovalProgramMismatch => {
            BankApplicationAttemptDenialKind::ElevationApprovalProgramMismatch
        }
        Query::ElevationCloseProgramMismatch => {
            BankApplicationAttemptDenialKind::ElevationCloseProgramMismatch
        }
        Query::MandatoryReviewProgramMismatch => {
            BankApplicationAttemptDenialKind::MandatoryReviewProgramMismatch
        }
        Query::WorkflowDefinitionAffinityMismatch => {
            BankApplicationAttemptDenialKind::WorkflowDefinitionAffinityMismatch
        }
        Query::WorkflowDefinitionAuthorityMismatch => {
            BankApplicationAttemptDenialKind::WorkflowDefinitionAuthorityMismatch
        }
        Query::WorkflowDefinitionIntentIdentityUnavailable => {
            BankApplicationAttemptDenialKind::WorkflowDefinitionIntentIdentityUnavailable
        }
        Query::WorkflowLineageUnavailable => {
            BankApplicationAttemptDenialKind::WorkflowLineageUnavailable
        }
        Query::WorkflowDefinitionCompilationUnavailable => {
            BankApplicationAttemptDenialKind::WorkflowDefinitionCompilationUnavailable
        }
        Query::WorkflowDefinitionMissingOperationInput => {
            BankApplicationAttemptDenialKind::WorkflowDefinitionMissingOperationInput
        }
        Query::CapturedOperationInputsNotSupported => {
            BankApplicationAttemptDenialKind::CapturedOperationInputsNotSupported
        }
        Query::WorkflowDefinitionSuperseded => {
            BankApplicationAttemptDenialKind::WorkflowDefinitionSuperseded
        }
        Query::WorkflowDefinitionRetired => {
            BankApplicationAttemptDenialKind::WorkflowDefinitionRetired
        }
        Query::WorkflowInstanceAffinityMismatch => {
            BankApplicationAttemptDenialKind::WorkflowInstanceAffinityMismatch
        }
        Query::WorkflowInstanceAuthorityMismatch => {
            BankApplicationAttemptDenialKind::WorkflowInstanceAuthorityMismatch
        }
        Query::WorkflowInstanceNotCompleted => {
            BankApplicationAttemptDenialKind::WorkflowInstanceNotCompleted
        }
        Query::WorkflowInstanceAlreadySucceeded => {
            BankApplicationAttemptDenialKind::WorkflowInstanceAlreadySucceeded
        }
        Query::WorkflowPredecessorOutputUnavailable => {
            BankApplicationAttemptDenialKind::WorkflowPredecessorOutputUnavailable
        }
        Query::WorkflowPredecessorOutputMismatch => {
            BankApplicationAttemptDenialKind::WorkflowPredecessorOutputMismatch
        }
        Query::WorkflowInstanceIntentIdentityUnavailable => {
            BankApplicationAttemptDenialKind::WorkflowInstanceIntentIdentityUnavailable
        }
        Query::WorkflowInstanceOriginalIntentNotFound => {
            BankApplicationAttemptDenialKind::WorkflowInstanceOriginalIntentNotFound
        }
        Query::WorkflowInstanceOriginalIntentAmbiguous => {
            BankApplicationAttemptDenialKind::WorkflowInstanceOriginalIntentAmbiguous
        }
        Query::WorkflowInstanceOriginalIntentUnavailable => {
            BankApplicationAttemptDenialKind::WorkflowInstanceOriginalIntentUnavailable
        }
        Query::WorkflowInstanceCapacityUnavailable => {
            BankApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable
        }
        Query::WorkflowLineageCapacityUnavailable => {
            BankApplicationAttemptDenialKind::WorkflowLineageCapacityUnavailable
        }
        Query::WorkflowInstanceCancelled => {
            BankApplicationAttemptDenialKind::WorkflowInstanceCancelled
        }
        Query::WorkflowInstanceTerminated => {
            BankApplicationAttemptDenialKind::WorkflowInstanceTerminated
        }
        Query::WorkflowInstanceCompleted => {
            BankApplicationAttemptDenialKind::WorkflowInstanceCompleted
        }
        Query::WorkflowOperationInOwnerCustody => {
            BankApplicationAttemptDenialKind::WorkflowOperationInOwnerCustody
        }
        Query::WorkflowInstanceHistoryUnavailable => {
            BankApplicationAttemptDenialKind::WorkflowInstanceHistoryUnavailable
        }
        Query::WorkflowInstanceMigrated => {
            BankApplicationAttemptDenialKind::WorkflowInstanceMigrated
        }
        Query::WorkflowInstanceMigrationUnmapped => {
            BankApplicationAttemptDenialKind::WorkflowInstanceMigrationUnmapped
        }
        Query::WorkflowInstanceDeadlineElapsed => {
            BankApplicationAttemptDenialKind::WorkflowInstanceDeadlineElapsed
        }
        Query::WorkflowTrustedTimeUnavailable => {
            BankApplicationAttemptDenialKind::WorkflowTrustedTimeUnavailable
        }
        Query::WorkflowInstanceEvidenceCapacityUnavailable => {
            BankApplicationAttemptDenialKind::WorkflowInstanceEvidenceCapacityUnavailable
        }
        Query::WorkflowTransitionAffinityMismatch => {
            BankApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch
        }
        Query::WorkflowTransitionAuthorityMismatch => {
            BankApplicationAttemptDenialKind::WorkflowTransitionAuthorityMismatch
        }
        Query::WorkflowTransitionAlreadySettled => {
            BankApplicationAttemptDenialKind::WorkflowTransitionAlreadySettled
        }
        Query::WorkflowTransitionNodeUnsupported => {
            BankApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported
        }
        Query::WorkflowTransitionOperationUnsettled => {
            BankApplicationAttemptDenialKind::WorkflowTransitionOperationUnsettled
        }
        Query::WorkflowTransitionIdentityUnavailable => {
            BankApplicationAttemptDenialKind::WorkflowTransitionIdentityUnavailable
        }
        Query::WorkflowConditionExpressionDenied => {
            BankApplicationAttemptDenialKind::WorkflowConditionExpressionDenied
        }
        Query::OperationBodyRequirementExpressionDenied => {
            BankApplicationAttemptDenialKind::OperationBodyRequirementExpressionDenied
        }
        Query::OperationBodyValueExpressionDenied => {
            BankApplicationAttemptDenialKind::OperationBodyValueExpressionDenied
        }
        Query::WorkflowAssessmentEvidenceIncomplete => {
            BankApplicationAttemptDenialKind::WorkflowAssessmentEvidenceIncomplete
        }
        Query::WorkflowAssessmentEvidenceMismatch => {
            BankApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch
        }
        Query::WorkflowApprovalPrincipalStale => {
            BankApplicationAttemptDenialKind::WorkflowApprovalPrincipalStale
        }
        Query::WorkflowApprovalGrantUnavailable => {
            BankApplicationAttemptDenialKind::WorkflowApprovalGrantUnavailable
        }
        Query::WorkflowApprovalExpired => BankApplicationAttemptDenialKind::WorkflowApprovalExpired,
        Query::WorkflowApprovalDelegationChanged => {
            BankApplicationAttemptDenialKind::WorkflowApprovalDelegationChanged
        }
        Query::WorkflowApprovalAuthorityDenied => {
            BankApplicationAttemptDenialKind::WorkflowApprovalAuthorityDenied
        }
        Query::ConflictingEffectStep => BankApplicationAttemptDenialKind::ConflictingEffectStep,
        Query::WorkflowHistoryReconstructionBudgetExceeded => {
            BankApplicationAttemptDenialKind::WorkflowHistoryReconstructionBudgetExceeded
        }
    }
}
