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
    ValueDecode(super::super::value_decode::WorthQueryValueDecodeDenial),
    /// Required committed native storage could not be admitted.
    StorageUnavailable(worth_relational::facade::storage::RelationalStorageUnavailable),
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
    /// neither outcome; [`expression`](WorthQueryApplicationAttemptDenial::expression)
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

/// Rich causes own their original box; native memory refusals stay inline.
/// Inspection preserves the original owner cause.
#[derive(Debug)]
enum AttemptDenialCause {
    IndexedLookup(Box<worth_relational::facade::indexes::BoundedEntityFieldLookupDenial>),
    OutputObservation(Box<crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop>),
    None,
    RelationalExecution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    NativeQueryRead(Box<worth_relational::facade::runtime::QueryReadExecutionStop>),
    RetainedSource(Box<super::retained_decision_facts::StoreDenial>),
    Expression(Box<worth_foundational::expression_api::ExpressionDenial>),
    Allocation(Box<worth_execution::ExecutionAllocationDenial>),
    RequestAuthority(
        Box<crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>,
    ),
}

impl WorthQueryApplicationAttemptDenial {
    pub(in crate::domain_computation) fn relational_execution_cause(
        &self,
    ) -> Option<worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        match self.cause {
            AttemptDenialCause::RelationalExecution(cause) => Some(cause),
            _ => None,
        }
    }

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

    /// The attempt's request lost its authority: it was cancelled, passed
    /// its deadline, or its authentication expired.
    pub(in crate::domain_computation::primary_graph) fn request_authority_lost(
        denial: crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationAttemptDenialKind::CurrentAuthorityDenied,
            subject: denial.subject().to_owned(),
            cause: AttemptDenialCause::RequestAuthority(Box::new(denial)),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn authorization_denied(
        kind: WorthQueryApplicationAttemptDenialKind,
        denial: crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    ) -> Self {
        Self {
            kind,
            subject: denial.subject().to_owned(),
            cause: AttemptDenialCause::RequestAuthority(Box::new(denial)),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn allocation_denied(
        subject: impl Into<String>,
        denial: worth_execution::ExecutionAllocationDenial,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationAttemptDenialKind::AllocationDenied,
            subject: subject.into(),
            cause: AttemptDenialCause::Allocation(Box::new(denial)),
        }
    }

    /// Exact physical owner refusal behind AllocationDenied; not graph authority.
    pub(in crate::domain_computation::primary_graph) fn retained_source_denied(
        kind: WorthQueryApplicationAttemptDenialKind,
        subject: &str,
        denial: super::retained_decision_facts::StoreDenial,
    ) -> Self {
        Self {
            kind,
            subject: subject.to_owned(),
            cause: AttemptDenialCause::RetainedSource(Box::new(denial)),
        }
    }
    pub fn source_retention_interruption(
        &self,
    ) -> Option<worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption>
    {
        match &self.cause {
            AttemptDenialCause::RetainedSource(denial) => match denial.as_ref() {
                super::retained_decision_facts::StoreDenial::RequestInterruption(interruption) => {
                    Some(*interruption)
                }
                _ => None,
            },
            _ => None,
        }
    }
    pub fn allocation_denial(&self) -> Option<&worth_execution::ExecutionAllocationDenial> {
        match &self.cause {
            AttemptDenialCause::Allocation(denial) => Some(denial.as_ref()),
            AttemptDenialCause::RelationalExecution(_) => None,
            AttemptDenialCause::RetainedSource(denial) => match denial.as_ref() {
                super::retained_decision_facts::StoreDenial::Allocation(denial) => Some(denial),
                _ => None,
            },
            _ => None,
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationAttemptDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// The original Relational read refusal, including non-request causes.
    pub fn native_query_read_stop(
        &self,
    ) -> Option<&worth_relational::facade::runtime::QueryReadExecutionStop> {
        match &self.cause {
            AttemptDenialCause::NativeQueryRead(stop) => Some(stop),
            _ => None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn into_native_query_read_stop(
        self,
    ) -> Option<Box<worth_relational::facade::runtime::QueryReadExecutionStop>> {
        match self.cause {
            AttemptDenialCause::NativeQueryRead(stop) => Some(stop),
            _ => None,
        }
    }

    /// The language denial behind a workflow condition or body requirement refusal.
    pub const fn expression(
        &self,
    ) -> Option<&worth_foundational::expression_api::ExpressionDenial> {
        match &self.cause {
            AttemptDenialCause::Expression(denial) => Some(denial),
            _ => None,
        }
    }

    /// The original authorization denial behind request-authority loss or
    /// approval readmission, including any retained native actor-source cause.
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

mod output_observation;

impl std::error::Error for WorthQueryApplicationAttemptDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.cause {
            AttemptDenialCause::IndexedLookup(denial) => Some(denial.as_ref()),
            AttemptDenialCause::OutputObservation(stop) => Some(stop.as_ref()),
            AttemptDenialCause::NativeQueryRead(stop) => Some(stop.as_ref()),
            AttemptDenialCause::RetainedSource(denial) => Some(denial.as_ref()),
            AttemptDenialCause::Allocation(denial) => Some(denial.as_ref()),
            AttemptDenialCause::RequestAuthority(denial) => Some(denial.as_ref()),
            AttemptDenialCause::RelationalExecution(_) => None,
            _ => None,
        }
    }
}

#[cfg(test)]
mod allocation_error_source_tests;

#[cfg(test)]
mod native_query_cause;

mod expression;
mod request_read;

mod capacity;
mod indexed_lookup;
