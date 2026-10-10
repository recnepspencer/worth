/// Why an output demand was refused while it was selected, admitted, or
/// advanced.
///
/// The denial's [`WorthQueryOutputDemandRecoveryPosture`] says whether asking
/// again can succeed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryOutputDemandDenialKind {
    /// The product root retained the precise delivery admission cause.
    ProductDelivery(Box<crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChangeDeliveryDenialKind>),
    /// Correspondence delivery retained its native non-success posture.
    CorrespondenceDelivery(Box<worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryStop>),
    /// The exact Bridge conditional refusal, retained before recovery grouping.
    BridgeConditional(Box<worth_runtime_bridge::facade::BridgeConditionalDenialKind>),
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
    /// The installed producer ran and its domain rejected the decision.
    ProducerDomainDenied,
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
