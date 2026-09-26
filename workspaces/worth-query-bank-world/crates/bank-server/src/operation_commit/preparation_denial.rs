//! Bank-owned denials raised before an application commit begins.

use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankApplicationAttemptDenialKind {
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
    DuplicateEffectKey,
    CandidateCapacityExceeded,
    CandidateReservationExceeded,
    RetainedEffectBytesExceeded,
    ExternalEffectPayloadProjectionRejected,
    ForeignConditionalDefinitionChange,
    DuplicateConditionalDefinitionChange,
    IncompleteEffectBasis,
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
    WorkflowInstanceAffinityMismatch,
    WorkflowInstanceAuthorityMismatch,
    WorkflowInstanceIntentIdentityUnavailable,
    WorkflowInstanceCapacityUnavailable,
    WorkflowInstanceCancelled,
    WorkflowInstanceMigrated,
    WorkflowInstanceMigrationUnmapped,
    WorkflowTransitionAffinityMismatch,
    WorkflowTransitionAuthorityMismatch,
    WorkflowTransitionAlreadySettled,
    WorkflowTransitionNodeUnsupported,
    WorkflowTransitionOperationUnsettled,
    WorkflowTransitionIdentityUnavailable,
    WorkflowAssessmentEvidenceIncomplete,
    WorkflowAssessmentEvidenceMismatch,
    WorkflowApprovalPrincipalStale,
    WorkflowApprovalGrantUnavailable,
    WorkflowApprovalExpired,
    WorkflowApprovalDelegationChanged,
    WorkflowApprovalAuthorityDenied,
    ConflictingEffectStep,
    WorkflowHistoryReconstructionBudgetExceeded,
    WorkflowTransitionCapacityExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BankCommitPreparationDenial {
    Application {
        kind: BankApplicationAttemptDenialKind,
    },
    InvalidProposalShape,
    AccountingRevisionOverflow,
}

impl From<WorthQueryApplicationAttemptDenial> for BankCommitPreparationDenial {
    fn from(denial: WorthQueryApplicationAttemptDenial) -> Self {
        Self::Application {
            kind: application_attempt_kind(denial.kind()),
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
        Query::ForeignApplication => BankApplicationAttemptDenialKind::ForeignApplication,
        Query::ProjectionAdmissionMismatch => {
            BankApplicationAttemptDenialKind::ProjectionAdmissionMismatch
        }
        Query::CurrentAuthorityDenied => BankApplicationAttemptDenialKind::CurrentAuthorityDenied,
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
        Query::DuplicateEffectKey => BankApplicationAttemptDenialKind::DuplicateEffectKey,
        Query::CandidateCapacityExceeded => {
            BankApplicationAttemptDenialKind::CandidateCapacityExceeded
        }
        Query::CandidateReservationExceeded => {
            BankApplicationAttemptDenialKind::CandidateReservationExceeded
        }
        Query::RetainedEffectBytesExceeded => {
            BankApplicationAttemptDenialKind::RetainedEffectBytesExceeded
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
        Query::WorkflowInstanceAffinityMismatch => {
            BankApplicationAttemptDenialKind::WorkflowInstanceAffinityMismatch
        }
        Query::WorkflowInstanceAuthorityMismatch => {
            BankApplicationAttemptDenialKind::WorkflowInstanceAuthorityMismatch
        }
        Query::WorkflowInstanceIntentIdentityUnavailable => {
            BankApplicationAttemptDenialKind::WorkflowInstanceIntentIdentityUnavailable
        }
        Query::WorkflowInstanceCapacityUnavailable => {
            BankApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable
        }
        Query::WorkflowInstanceCancelled => {
            BankApplicationAttemptDenialKind::WorkflowInstanceCancelled
        }
        Query::WorkflowInstanceMigrated => {
            BankApplicationAttemptDenialKind::WorkflowInstanceMigrated
        }
        Query::WorkflowInstanceMigrationUnmapped => {
            BankApplicationAttemptDenialKind::WorkflowInstanceMigrationUnmapped
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
        Query::WorkflowTransitionCapacityExceeded => {
            BankApplicationAttemptDenialKind::WorkflowTransitionCapacityExceeded
        }
    }
}
