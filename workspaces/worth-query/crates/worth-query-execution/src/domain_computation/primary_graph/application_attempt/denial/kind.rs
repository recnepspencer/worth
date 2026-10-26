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
    /// The original request refused an admitted field decode; completion is vetoed.
    ValueDecode(
        crate::domain_computation::primary_graph::value_decode::WorthQueryValueDecodeDenial,
    ),
    /// Required committed native storage could not be admitted.
    StorageUnavailable,
    ForeignPhase,
    ExecutionRequest(crate::domain_computation::WorthQueryProviderSessionDenialKind),
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
    /// Required original correction interpretation could not be retained.
    CorrectionCaptureDenied,
    DecisionFactBudgetExceeded,
    /// Fresh inline payload backing was refused by its allocation owner.
    AllocationDenied,
    /// Checked representation or ordering failure while retaining source facts.
    RetainedSourceStorageDenied,
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
    /// The role's cardinality differs from the one the installed contract
    /// records for it. A typed use cannot disagree with its contract, so this
    /// guards the installed form as defense in depth.
    OutputRoleCardinalityMismatch,
    DuplicateEffectKey,
    ConflictingEffectStep,
    CandidateCapacityExceeded,
    ExternalEffectPayloadProjectionRejected,
    ForeignConditionalDefinitionChange,
    DuplicateConditionalDefinitionChange,
    IncompleteEffectBasis,
    /// A write of a unique field names a value another live entity holds,
    /// or the program writes that value twice. Deleting the holder in the
    /// same program does not free it.
    UniqueValueTaken,
    /// A unique field's equality index could not answer the lookup its write
    /// must observe.
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
    /// Stored guarded operation meaning lacks exactly one input source.
    WorkflowDefinitionMissingOperationInput,
    /// Captured-target meaning cannot execute before its native input producer exists.
    CapturedOperationInputsNotSupported,
    /// A start named a definition its branch has since superseded. A retry
    /// of a recorded start still replays.
    WorkflowDefinitionSuperseded,
    /// A start named a definition of a retired lineage. A retry of a
    /// recorded start still replays.
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
    WorkflowInstanceTerminated,
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
    /// neither outcome; [`expression`](super::WorthQueryApplicationAttemptDenial::expression)
    /// carries the language denial.
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
}
