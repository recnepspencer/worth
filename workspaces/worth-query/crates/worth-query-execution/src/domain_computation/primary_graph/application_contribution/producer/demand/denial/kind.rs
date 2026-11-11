/// Why an output demand was refused while it was selected, admitted, or
/// advanced.
///
/// The denial's [`WorthQueryOutputDemandRecoveryPosture`] says whether asking
/// again can succeed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryOutputDemandDenialKind {
    ConflictingIntent { target: worth_relational::facade::transactions::RecordRef, earlier: usize, later: usize },
    AuthoredStageRefused { ordinal: usize, binding: &'static str },
    /// The original unpublished record refused its native recovery visit.
    WorldRecovery(worth_runtime_world::facade::RuntimeWorldRecoveryDenial),
    /// The original settled effect could not publish its World successor.
    RecoveryPublication(worth_runtime_world::facade::NoEffectCause),
    /// A fresh admitted read could not deliver the original published receipt.
    PublicationReceipt(crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenialKind),
    /// The current control owner is in use. No source result was handed off.
    CurrentControlBusy,
    PredecessorRead(Box<super::super::super::predecessor_reads::WorthQueryPredecessorReadDenial>),
    /// Native current roster/presence or storage refused this exact publication read.
    CurrentPublication(Box<worth_relational::facade::transactions::TransactionCommitError>),
    ManagedIndexSourceDenied,
    /// The producer returned actual managed cleanup custody; ordinary rerun is not authorized.
    ManagedCleanupRequired(crate::domain_computation::WorthQueryMutationCleanupFailureKind),
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
        crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionRefusalKind,
    ),
    /// Re-admitting the demand's original source query could not resolve its scope.
    SourceScope(crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind),
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
    /// Managed query dependencies have no supported fact-expansion path yet.
    ManagedSourceExpansionUnsupported,
    /// No installed producer covers this output family and applicability.
    MissingApplicableProducer,
    /// More than one installed producer covers this output family and
    /// applicability.
    AmbiguousApplicableProducer,
    /// The selected producer, its applicability, or its output binding is no
    /// longer installed.
    ProducerUnavailable,
    /// The producer attempted a field write without its completed decision observation.
    UnobservedWrite(crate::domain_computation::primary_graph::WorthQueryApplicationUnobservedWrite),
    /// The installed producer ran and its domain rejected the decision.
    ProducerDomainDenied,
    /// An installed connection rejected discovery input or its computed demands.
    DiscoveryConnection(Box<worth_query_declaration::facade::application_program::WorthQueryRequiredOutputConnectionDenial>),
    /// The selected original source could not supply a concrete resource quote.
    /// The admitted request lost its operation authorization before publication.
    RequestAuthorization(
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind,
    ),
    /// The product branch could not be selected; the admission denial says why.
    ProductSelection(crate::basis::WorthQueryProductBranchAdmissionRefusalKind),
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
    /// The next handler start exceeds this exact route's edit policy.
    ProducerRouteRoundBudgetExceeded { source: worth_relational::facade::identity::EntityId, started: usize, limit: usize },
    /// The next handler start exceeds the complete edit attempt's policy.
    ProducerAttemptRoundBudgetExceeded { source: worth_relational::facade::identity::EntityId, started: usize, limit: usize },
    /// The producer's retained output is larger than the demand allows.
    RetentionBudgetExceeded,
    /// The branch had no capacity to publish the output.
    PublicationCapacityExceeded,
    /// The demand, or its dependent connection, belongs to another runtime,
    /// schema binding, or program.
    ForeignDemand,
    /// An internal required successor lost its admitted demand or execution provenance.
    RequiredSuccessorStateUnavailable,
    /// A dependent output's parent settlement belongs to another program,
    /// feature, or branch position.
    ForeignSettlement,
    /// A retained verification requirement names a fact absent from its retained body.
    RetainedVerificationFactUnavailable(crate::domain_computation::primary_graph::provider::RebaseVerificationReason),
    /// The exact retained provenance fact that disagrees with its real owner.
    SettlementProvenance(super::WorthQueryOutputSettlementProvenanceDenial),
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
