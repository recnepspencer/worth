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
    AmbiguousRelation,
    UndeclaredEffect,
    ForeignEffectTarget,
    DuplicateEffectKey,
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
}

#[derive(Debug)]
pub struct WorthQueryApplicationAttemptDenial {
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: String,
}

impl WorthQueryApplicationAttemptDenial {
    pub(super) fn new(
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
