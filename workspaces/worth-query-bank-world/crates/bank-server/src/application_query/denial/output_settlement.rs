//! One exhaustive descriptive projection of Query output-settlement refusals.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BankApplicationOutputSettlementDenialKind {
    ConflictingIntent {
        target: worth_query_host::facade::primary_graph::RecordRef,
        earlier: usize,
        later: usize,
    },
    AuthoredStageRefused {
        ordinal: usize,
        binding: &'static str,
    },
    DurableStateAdmission(worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial),
    ObligationPublication(worth_query_host::facade::primary_graph::QueryObligationDenial),
    WorldRecovery(worth_query_host::facade::primary_graph::RuntimeWorldRecoveryDenial),
    RecoveryPublication(worth_query_host::facade::primary_graph::NoEffectCause),
    PublicationReceipt(worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolutionDenialKind),
    CurrentControlBusy,
    PredecessorRead(Box<worth_query_host::facade::application_contribution::WorthQueryPredecessorReadDenial>),
    CurrentPublication(Box<worth_query_host::facade::primary_graph::TransactionCommitError>),
    RetainedVerificationFactUnavailable(Box<worth_query_host::facade::primary_graph::RebaseVerificationReason>),
    SettlementProvenance(worth_query_host::facade::primary_graph::WorthQueryOutputSettlementProvenanceDenial),
    ManagedIndexSourceDenied,
    ManagedCleanupRequired(worth_query_host::facade::primary_graph::WorthQueryMutationCleanupFailureKind),
    RelationalExecution(worth_query_host::facade::primary_graph::RelationalExecutionDenialCause),
    BridgeConditional(Box<worth_query_host::facade::primary_graph::BridgeConditionalDenialKind>),
    CorrespondenceDelivery(Box<worth_query_host::facade::primary_graph::BridgeCorrespondenceDeliveryStop>),
    ProductDelivery(Box<worth_query_host::facade::product::WorthQueryPerformedRelationalProductChangeDeliveryDenialKind>),
    ExecutionRequest(
        worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial,
    ),
    SourceQueryInstallation(BankApplicationQueryInstallationDenialKind),
    SourcePrincipal(WorthQueryPrincipalResolutionDenialKind),
    SourceScope(BankEntityResolutionDenialKind),
    SourceQueryAdmission(BankApplicationQueryAdmissionDenialKind),
    SourceQueryExecution(BankApplicationOneShotDenialKind),
    ForeignSource,
    MissingApplicableProducer,
    AmbiguousApplicableProducer,
    ProducerUnavailable,
    ProducerDomainDenied,
    DiscoveryConnection(
        Box<worth_query_host::facade::primary_graph::WorthQueryRequiredOutputConnectionDenial>,
    ),
    RequestAuthorization(BankAuthorizationDenialKind),
    ProductSelection(BankProductSelectionDenialKind),
    SchedulingRejected,
    SchedulingDeferred,
    PublicationStale,
    NoEffect,
    Superseded,
    Cancelled,
    TimedOut,
    WorkBudgetExceeded,
    ProducerRouteRoundBudgetExceeded {
        source: worth_query_host::facade::application_invariants::EntityId,
        started: usize,
        limit: usize,
    },
    ProducerAttemptRoundBudgetExceeded {
        source: worth_query_host::facade::application_invariants::EntityId,
        started: usize,
        limit: usize,
    },
    RetentionBudgetExceeded,
    PublicationCapacityExceeded,
    ForeignDemand,
    RequiredSuccessorStateUnavailable,
    ForeignSettlement,
    RetainedBasisUnavailable,
    Closed,
    DuplicatePerformedSource,
    IncompleteDependencyCoverage,
    ManagedSourceExpansionUnsupported,
    UnobservedWrite(worth_query_host::facade::primary_graph::WorthQueryApplicationUnobservedWrite),
}

impl From<WorthQueryOutputDemandDenialKind> for BankApplicationOutputSettlementDenialKind {
    fn from(kind: WorthQueryOutputDemandDenialKind) -> Self {
        use BankApplicationOutputSettlementDenialKind as Bank;
        use WorthQueryOutputDemandDenialKind as Query;
        match kind {
            Query::ConflictingIntent { target, earlier, later } => {
                Bank::ConflictingIntent { target, earlier, later }
            }
            Query::AuthoredStageRefused { ordinal, binding } => {
                Bank::AuthoredStageRefused { ordinal, binding }
            }
            Query::ObligationPublication(cause) => Bank::ObligationPublication(cause),
            Query::WorldRecovery(cause) => Bank::WorldRecovery(cause),
            Query::RecoveryPublication(cause) => Bank::RecoveryPublication(cause),
            Query::PublicationReceipt(cause) => Bank::PublicationReceipt(cause),
            Query::CurrentControlBusy => Bank::CurrentControlBusy,
            Query::PredecessorRead(cause) => Bank::PredecessorRead(cause),
            Query::CurrentPublication(cause) => Bank::CurrentPublication(cause),
            Query::RetainedVerificationFactUnavailable(cause) => Bank::RetainedVerificationFactUnavailable(Box::new(cause)),
            Query::SettlementProvenance(cause) => Bank::SettlementProvenance(cause),
            Query::ManagedIndexSourceDenied => Bank::ManagedIndexSourceDenied,
            Query::ManagedCleanupRequired(kind) => Bank::ManagedCleanupRequired(kind),
            Query::BridgeConditional(cause) => Bank::BridgeConditional(cause),
            Query::CorrespondenceDelivery(cause) => Bank::CorrespondenceDelivery(cause),
            Query::ProductDelivery(cause) => Bank::ProductDelivery(cause),
            Query::ExecutionRequest(cause) => Bank::ExecutionRequest(cause),
            Query::SourceQueryInstallation(kind) => {
                Bank::SourceQueryInstallation(query_installation(kind))
            }
            Query::SourcePrincipal(kind) => Bank::SourcePrincipal(WorthQueryPrincipalResolutionDenialKind::Refused(kind)),
            Query::SourceScope(kind) => {
                Bank::SourceScope(BankEntityResolutionDenial::from_query(worth_query_host::facade::primary_graph::WorthQueryEntityResolutionDenialKind::Refused(kind)).kind())
            }
            Query::SourceQueryAdmission(kind) => Bank::SourceQueryAdmission(admission(kind)),
            Query::SourceQueryExecution(kind) => Bank::SourceQueryExecution(one_shot(kind)),
            Query::ForeignSource => Bank::ForeignSource,
            Query::MissingApplicableProducer => Bank::MissingApplicableProducer,
            Query::AmbiguousApplicableProducer => Bank::AmbiguousApplicableProducer,
            Query::ProducerUnavailable => Bank::ProducerUnavailable,
            Query::ProducerDomainDenied => Bank::ProducerDomainDenied,
            Query::DiscoveryConnection(cause) => Bank::DiscoveryConnection(cause),
            Query::RequestAuthorization(kind) => Bank::RequestAuthorization(authorization(worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(kind))),
            Query::ProductSelection(refusal) => Bank::ProductSelection(product_selection(worth_query_host::facade::primary_graph::WorthQueryProductBranchAdmissionDenial::Refused(refusal))),
            Query::SchedulingRejected => Bank::SchedulingRejected,
            Query::SchedulingDeferred => Bank::SchedulingDeferred,
            Query::PublicationStale => Bank::PublicationStale,
            Query::NoEffect => Bank::NoEffect,
            Query::Superseded => Bank::Superseded,
            Query::Cancelled => Bank::Cancelled,
            Query::TimedOut => Bank::TimedOut,
            Query::WorkBudgetExceeded => Bank::WorkBudgetExceeded,
            Query::ProducerRouteRoundBudgetExceeded { source, started, limit } => {
                Bank::ProducerRouteRoundBudgetExceeded { source, started, limit }
            }
            Query::ProducerAttemptRoundBudgetExceeded { source, started, limit } => {
                Bank::ProducerAttemptRoundBudgetExceeded { source, started, limit }
            }
            Query::RetentionBudgetExceeded => Bank::RetentionBudgetExceeded,
            Query::PublicationCapacityExceeded => Bank::PublicationCapacityExceeded,
            Query::ForeignDemand => Bank::ForeignDemand,
            Query::RequiredSuccessorStateUnavailable => Bank::RequiredSuccessorStateUnavailable,
            Query::ForeignSettlement => Bank::ForeignSettlement,
            Query::RetainedBasisUnavailable => Bank::RetainedBasisUnavailable,
            Query::Closed => Bank::Closed,
            Query::DuplicatePerformedSource => Bank::DuplicatePerformedSource,
            Query::IncompleteDependencyCoverage => Bank::IncompleteDependencyCoverage,
            Query::DurableStateAdmission(cause) => Bank::DurableStateAdmission(cause),
            Query::ManagedSourceExpansionUnsupported => Bank::ManagedSourceExpansionUnsupported,
            Query::UnobservedWrite(cause) => Bank::UnobservedWrite(cause),
        }
    }
}
