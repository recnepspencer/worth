//! One exhaustive descriptive projection of Query output-settlement refusals.
use super::*;
impl From<WorthQueryOutputDemandDenialKind> for BankApplicationOutputSettlementDenialKind {
    fn from(kind: WorthQueryOutputDemandDenialKind) -> Self {
        use BankApplicationOutputSettlementDenialKind as Bank;
        use WorthQueryOutputDemandDenialKind as Query;
        match kind {
            Query::WorldRecovery(cause) => Bank::WorldRecovery(cause),
            Query::RecoveryPublication(cause) => Bank::RecoveryPublication(cause),
            Query::PublicationReceipt(cause) => Bank::PublicationReceipt(cause),
            Query::CurrentControlBusy => Bank::CurrentControlBusy,
            cause @ Query::RetainedVerificationFactUnavailable(_) => Bank::RetainedVerificationFactUnavailable(Box::new(cause)),
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
            Query::RetentionBudgetExceeded => Bank::RetentionBudgetExceeded,
            Query::PublicationCapacityExceeded => Bank::PublicationCapacityExceeded,
            Query::ForeignDemand => Bank::ForeignDemand,
            Query::RequiredSuccessorStateUnavailable => Bank::RequiredSuccessorStateUnavailable,
            Query::ForeignSettlement => Bank::ForeignSettlement,
            Query::RetainedBasisUnavailable => Bank::RetainedBasisUnavailable,
            Query::Closed => Bank::Closed,
            Query::DuplicatePerformedSource => Bank::DuplicatePerformedSource,
            Query::IncompleteDependencyCoverage => Bank::IncompleteDependencyCoverage,
            Query::ManagedSourceExpansionUnsupported => Bank::ManagedSourceExpansionUnsupported,
            Query::UnobservedWrite(cause) => Bank::UnobservedWrite(cause),
        }
    }
}
