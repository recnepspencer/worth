//! Bank-owned denials raised before an application commit begins.

use worth_query_host::facade::primary_graph::RelationalStorageUnavailable;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use worth_query_host::facade::runtime::{ExecutionAllocationDenial, WorkCeilingDenial};

mod application_attempt_kind;
use application_attempt_kind::application_attempt_kind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankApplicationAttemptDenialKind {
    ValueDecode(worth_query_host::facade::primary_graph::WorthQueryValueDecodeDenial),
    ForeignPhase,
    StorageUnavailable,
    ExecutionRequest(worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind),
    EffectExecution(WorkCeilingDenial),
    Cancelled,
    DeadlineExceeded,
    ForeignApplication,
    ProjectionAdmissionMismatch,
    CurrentAuthorityDenied,
    OutsideRealizedReadScope,
    UndeclaredDecisionRead,
    UnobservedWrite,
    StaleEntityIdentity,
    MissingAuthoritativeFact,
    InvalidAuthoritativeValue,
    IncompleteDecisionReadSet,
    DecisionDependencyMismatch,
    DecisionFactBudgetExceeded,
    AllocationDenied,
    RetainedSourceStorageDenied,
    CorrectionCaptureDenied,
    SourceDisclosureCaptureDenied,
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
    StorageUnavailable(RelationalStorageUnavailable),
    Application {
        kind: BankApplicationAttemptDenialKind,
        /// Physical allocation refusal, including its original checked quote.
        allocation: Option<ExecutionAllocationDenial>,
    },
}

impl BankCommitPreparationDenial {
    pub const fn application_kind(&self) -> Option<BankApplicationAttemptDenialKind> {
        match self {
            Self::StorageUnavailable(_) => {
                Some(BankApplicationAttemptDenialKind::StorageUnavailable)
            }
            Self::Application { kind, .. } => Some(*kind),
        }
    }
    pub const fn storage_unavailable(&self) -> Option<&RelationalStorageUnavailable> {
        match self {
            Self::StorageUnavailable(failure) => Some(failure),
            _ => None,
        }
    }
}

impl From<WorthQueryApplicationAttemptDenial> for BankCommitPreparationDenial {
    fn from(denial: WorthQueryApplicationAttemptDenial) -> Self {
        if let Some(failure) = denial.storage_unavailable() {
            return Self::StorageUnavailable(*failure);
        }
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

#[cfg(test)]
#[path = "preparation_denial_tests.rs"]
mod tests;
