/// Why an output demand was refused while it was selected, admitted, or
/// advanced.
///
/// The denial's [`WorthQueryOutputDemandRecoveryPosture`] says whether asking
/// again can succeed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOutputDemandDenialKind {
    /// The advancement could not enter its host-owned execution request.
    ExecutionRequest(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    /// The original source query binding is no longer installed as admitted.
    SourceQueryInstallation(
        worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind,
    ),
    /// Re-admitting the demand's original source query could not resolve its principal.
    SourcePrincipal(
        crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenialKind,
    ),
    /// Re-admitting the demand's original source query could not resolve its scope.
    SourceScope(crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind),
    /// The retained source query was refused before its read.
    SourceQueryAdmission(
        crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind,
    ),
    /// The retained source query was refused during its one-shot read.
    SourceQueryExecution(
        crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind,
    ),
    /// The observed source belongs to another runtime, schema binding, product
    /// commit, or product occurrence, or is not a product-branch source of the
    /// family's installed query.
    ForeignSource,
    /// No installed producer covers this output family and applicability.
    MissingApplicableProducer,
    /// More than one installed producer covers this output family and
    /// applicability.
    AmbiguousApplicableProducer,
    /// The selected producer, its applicability, or its output binding is no
    /// longer installed.
    ProducerUnavailable,
    /// The admitted request lost its operation authorization before publication.
    RequestAuthorization(
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind,
    ),
    /// The product branch could not be selected; the admission denial says why.
    ProductSelection(crate::basis::WorthQueryProductBranchAdmissionDenial),
    /// Scheduling the producer was refused and will not succeed as asked.
    SchedulingRejected,
    /// Scheduling the producer is blocked by a temporary limit. Retry later.
    SchedulingDeferred,
    /// The product branch or active program moved before the output was
    /// published.
    PublicationStale,
    /// The producer's signal found nothing to recompute, so no output was
    /// produced.
    NoEffect,
    /// The source no longer matches the admitted demand; a newer source
    /// replaces it.
    Superseded,
    /// Publishing the output was canceled.
    Cancelled,
    /// Publishing the output reached its deadline.
    TimedOut,
    /// The producer needs more work than the demand allows.
    WorkBudgetExceeded,
    /// The producer's retained output is larger than the demand allows.
    RetentionBudgetExceeded,
    /// The branch had no capacity to publish the output.
    PublicationCapacityExceeded,
    /// The demand, or its dependent connection, belongs to another runtime,
    /// schema binding, or program.
    ForeignDemand,
    /// A dependent output's parent settlement belongs to another program,
    /// feature, or branch position.
    ForeignSettlement,
    /// A retained or restored output lacks the dependency facts needed to reuse
    /// it.
    IncompleteDependencyCoverage,
    /// The retained basis the demand relies on can no longer be selected.
    RetainedBasisUnavailable,
    /// The demand was closed; it no longer has an interest to advance.
    Closed,
    /// The performed source is absent or was already consumed by another
    /// demand.
    DuplicatePerformedSource,
}

/// Whether a refused output demand can succeed if asked again.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOutputDemandRecoveryPosture {
    /// Asking again, later or with a larger budget, can succeed.
    Retryable,
    /// Asking again with the same inputs will be refused again.
    Terminal,
}

/// A refusal to select, admit, or advance an output demand, with the subject it
/// names (usually the output family or producer).
///
/// Match on [`kind`](Self::kind) and check
/// [`recovery_posture`](Self::recovery_posture) before retrying; the subject is
/// for diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputDemandDenial {
    kind: WorthQueryOutputDemandDenialKind,
    commit_denial_kind:
        Option<crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind>,
    commit_execution: Option<
        Box<(
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
            Result<
                crate::domain_computation::WorthQueryProviderSessionDenialKind,
                crate::domain_computation::WorthQueryProviderSessionControlStopKind,
            >,
        )>,
    >,
    subject: std::borrow::Cow<'static, str>,
    pub(in crate::domain_computation::primary_graph) recovery_posture:
        WorthQueryOutputDemandRecoveryPosture,
}

impl WorthQueryOutputDemandDenial {
    pub const fn kind(&self) -> WorthQueryOutputDemandDenialKind {
        self.kind
    }

    /// The original pre-effect commit refusal, when this demand ran a producer.
    pub const fn commit_denial_kind(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind>
    {
        self.commit_denial_kind
    }

    pub(in crate::domain_computation::primary_graph) fn with_commit_denial_kind(
        mut self,
        kind: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind,
    ) -> Self {
        self.commit_denial_kind = Some(kind);
        self
    }

    /// Typed execution evidence beside the historical producer failure kind.
    pub fn commit_execution_denial(
        &self,
    ) -> Option<(
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
        Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    )> {
        self.commit_execution.as_deref().copied()
    }

    pub(in crate::domain_computation::primary_graph) fn with_commit_execution_denial(
        mut self,
        evidence: Option<(
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
            Result<
                crate::domain_computation::WorthQueryProviderSessionDenialKind,
                crate::domain_computation::WorthQueryProviderSessionControlStopKind,
            >,
        )>,
    ) -> Self {
        self.commit_execution = evidence.map(Box::new);
        self
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub const fn recovery_posture(&self) -> WorthQueryOutputDemandRecoveryPosture {
        self.recovery_posture
    }

    pub(in crate::domain_computation::primary_graph) fn new(
        kind: WorthQueryOutputDemandDenialKind,
        subject: impl Into<std::borrow::Cow<'static, str>>,
    ) -> Self {
        Self {
            kind,
            commit_denial_kind: None,
            commit_execution: None,
            subject: subject.into(),
            recovery_posture: kind.default_recovery_posture(),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn product_selection(
        denial: crate::basis::WorthQueryProductBranchAdmissionDenial,
        subject: impl Into<std::borrow::Cow<'static, str>>,
    ) -> Self {
        Self::new(
            WorthQueryOutputDemandDenialKind::ProductSelection(denial),
            subject,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn with_recovery_posture(
        mut self,
        recovery_posture: WorthQueryOutputDemandRecoveryPosture,
    ) -> Self {
        self.recovery_posture = recovery_posture;
        self
    }
}

impl std::fmt::Display for WorthQueryOutputDemandDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "output demand denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryOutputDemandDenial {}

impl WorthQueryOutputDemandDenial {
    /// A typed refusal before the caller pass enters its request.
    pub fn request_admission(
        cause: crate::domain_computation::primary_graph::WorthQueryAdvancementDenial,
    ) -> Self {
        Self::new(
            WorthQueryOutputDemandDenialKind::ExecutionRequest(cause),
            "request admission",
        )
    }
}
