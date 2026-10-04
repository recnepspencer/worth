/// Why an application attempt was refused while reading its decision basis or
/// authoring its candidate.
///
/// Nothing was committed. The families are: foreign or stale inputs (wrong
/// application, current authority lost, source retired or changed, stale entity
/// identity); decision-read
/// violations (undeclared, outside scope, incomplete, over budget, precondition
/// mismatch); effect violations (undeclared effect, invalid value, foreign
/// target, output-role mismatch); candidate capacity limits; lane mismatches,
/// where the operation needs a delegation, capability, elevation, or review lane;
/// and workflow refusals for definitions, instances, transitions, assessments,
/// and approvals.
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
    /// A start named a definition its branch has since superseded. A retry
    /// of a recorded start still replays.
    WorkflowDefinitionSuperseded,
    /// A start named a definition of a retired lineage. A retry of a
    /// recorded start still replays.
    WorkflowDefinitionRetired,
    WorkflowInstanceAffinityMismatch,
    WorkflowInstanceAuthorityMismatch,
    WorkflowInstanceIntentIdentityUnavailable,
    /// The instance's lineage has taken its installed number of steps, or
    /// its retained progress would pass the installed byte budget. Recorded
    /// steps still replay and the instance can still be cancelled; ending
    /// other instances frees nothing for it.
    WorkflowInstanceCapacityUnavailable,
    /// The lineage already holds its installed number of live instances, so
    /// a new start is refused. A retry of a recorded start still replays,
    /// and cancelling or completing an instance frees room.
    WorkflowLineageCapacityUnavailable,
    /// An explicit cancellation or program adoption cancelled the instance;
    /// no request can advance it.
    WorkflowInstanceCancelled,
    /// The instance completed; nothing is left to cancel.
    WorkflowInstanceCompleted,
    /// An external operation the instance ran committed into its owner's
    /// custody and has not settled. The instance cannot be cancelled until
    /// the owner's receipt is accepted; the cancellation then reports it.
    WorkflowOperationInOwnerCustody,
    /// The effects the instance, or a source it was migrated from, performed
    /// cannot be read within their retained bounds.
    WorkflowInstanceHistoryUnavailable,
    /// An explicit migration ended the instance; its successor continues.
    WorkflowInstanceMigrated,
    /// The migration target cannot lawfully continue the instance from the
    /// requested node: a performed effect is unmapped or would run again, or
    /// a node it would run consumes a result the successor cannot produce.
    WorkflowInstanceMigrationUnmapped,
    /// The total deadline a definition in the instance's lineage declared
    /// has elapsed on the installed clock. The instance takes no further
    /// step, migration, or fork continuation; it can still be cancelled.
    WorkflowInstanceDeadlineElapsed,
    /// The installed clock could not be read, so a deadline cannot be shown
    /// to lie ahead and the step is refused.
    WorkflowTrustedTimeUnavailable,
    /// The assessment evidence the instance's lineage retains would exceed
    /// the installed evidence ceiling. Recorded steps still replay, and the
    /// instance can still be cancelled or navigate without new evidence.
    WorkflowInstanceEvidenceCapacityUnavailable,
    WorkflowHistoryReconstructionBudgetExceeded,
    WorkflowTransitionAffinityMismatch,
    WorkflowTransitionAuthorityMismatch,
    WorkflowTransitionAlreadySettled,
    WorkflowTransitionNodeUnsupported,
    /// Back or migration would abandon an operation whose approval has not
    /// been consumed by a receipted settlement.
    WorkflowTransitionOperationUnsettled,
    WorkflowTransitionIdentityUnavailable,
    /// A condition's expression denied over its current operand values, or a
    /// value could not be read as its declared type. The expression decided
    /// neither outcome; [`expression`](WorthQueryApplicationAttemptDenial::expression)
    /// carries the language denial.
    WorkflowConditionExpressionDenied,
    WorkflowAssessmentEvidenceIncomplete,
    WorkflowAssessmentEvidenceMismatch,
    WorkflowApprovalPrincipalStale,
    WorkflowApprovalGrantUnavailable,
    WorkflowApprovalExpired,
    WorkflowApprovalDelegationChanged,
    WorkflowApprovalAuthorityDenied,
}

/// A typed refusal of an application attempt before commit, with the subject it
/// names (usually the operation, entity, field, or relation involved).
///
/// Nothing was committed. Match on [`kind`](Self::kind); the subject is for
/// diagnostics.
#[derive(Debug)]
pub struct WorthQueryApplicationAttemptDenial {
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: String,
    cause: AttemptDenialCause,
}

/// The typed denial a kind carries, when it has one.
#[derive(Debug)]
enum AttemptDenialCause {
    None,
    Expression(worth_foundational::expression_api::ExpressionDenial),
    RequestAuthority(
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    ),
}

impl WorthQueryApplicationAttemptDenial {
    pub(in crate::domain_computation::primary_graph) fn new(
        kind: WorthQueryApplicationAttemptDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
            cause: AttemptDenialCause::None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn condition_expression(
        subject: impl Into<String>,
        denial: worth_foundational::expression_api::ExpressionDenial,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationAttemptDenialKind::WorkflowConditionExpressionDenied,
            subject: subject.into(),
            cause: AttemptDenialCause::Expression(denial),
        }
    }

    /// The attempt's request lost its authority: it was cancelled, passed
    /// its deadline, or its authentication expired.
    pub(in crate::domain_computation::primary_graph) fn request_authority_lost(
        denial: crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationAttemptDenialKind::CurrentAuthorityDenied,
            subject: denial.subject().to_owned(),
            cause: AttemptDenialCause::RequestAuthority(denial),
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationAttemptDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// The language denial behind `WorkflowConditionExpressionDenied`.
    pub const fn expression(
        &self,
    ) -> Option<&worth_foundational::expression_api::ExpressionDenial> {
        match &self.cause {
            AttemptDenialCause::Expression(denial) => Some(denial),
            _ => None,
        }
    }

    /// The request's own stop behind `CurrentAuthorityDenied`, when the
    /// request rather than the attempt lost its authority.
    pub const fn request_authority(
        &self,
    ) -> Option<&crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    {
        match &self.cause {
            AttemptDenialCause::RequestAuthority(denial) => Some(denial),
            _ => None,
        }
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
