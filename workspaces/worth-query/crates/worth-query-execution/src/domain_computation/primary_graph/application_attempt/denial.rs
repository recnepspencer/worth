#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationAttemptDenialKind {
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
    ConflictingEffectStep,
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
    /// Program adoption cancelled the instance; no request can advance it.
    WorkflowInstanceCancelled,
    /// An explicit migration ended the instance; its successor continues.
    WorkflowInstanceMigrated,
    /// The migration target cannot lawfully continue the instance from the
    /// requested node: a performed effect is unmapped or would run again, or
    /// a node it would run consumes a result the successor cannot produce.
    WorkflowInstanceMigrationUnmapped,
    WorkflowHistoryReconstructionBudgetExceeded,
    WorkflowTransitionAffinityMismatch,
    WorkflowTransitionAuthorityMismatch,
    WorkflowTransitionAlreadySettled,
    WorkflowTransitionNodeUnsupported,
    /// Back or migration would abandon an operation whose approval has not
    /// been consumed by a receipted settlement.
    WorkflowTransitionOperationUnsettled,
    WorkflowTransitionIdentityUnavailable,
    WorkflowTransitionCapacityExceeded,
    WorkflowAssessmentEvidenceIncomplete,
    WorkflowAssessmentEvidenceMismatch,
    WorkflowApprovalPrincipalStale,
    WorkflowApprovalGrantUnavailable,
    WorkflowApprovalExpired,
    WorkflowApprovalDelegationChanged,
    WorkflowApprovalAuthorityDenied,
}

#[derive(Debug)]
pub struct WorthQueryApplicationAttemptDenial {
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: String,
}

impl WorthQueryApplicationAttemptDenial {
    pub(in crate::domain_computation::primary_graph) fn new(
        kind: WorthQueryApplicationAttemptDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationAttemptDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryApplicationAttemptDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application attempt {:?}: {}",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryApplicationAttemptDenial {}
