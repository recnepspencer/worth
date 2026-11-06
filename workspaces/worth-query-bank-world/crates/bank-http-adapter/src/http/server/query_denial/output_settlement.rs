//! HTTP instructions for the original output-settlement refusal.
use super::*;
pub(super) fn output_settlement(kind: BankApplicationOutputSettlementDenialKind) -> BankHttpDenial {
    use BankApplicationOutputSettlementDenialKind as Settlement;
    match kind {
        Settlement::CurrentControlBusy => unavailable(),
        Settlement::WorldRecovery(_)
        | Settlement::RecoveryPublication(_)
        | Settlement::PublicationReceipt(_)
        | Settlement::RetainedVerificationFactUnavailable(_)
        | Settlement::SettlementProvenance(_)
        | Settlement::ManagedIndexSourceDenied => internal_denied(),
        Settlement::ManagedCleanupRequired(_) => {
            BankHttpDenial::new(Kind::ManagedCleanupRequired, Next::ContactOperator)
        }
        Settlement::BridgeConditional(_)
        | Settlement::CorrespondenceDelivery(_)
        | Settlement::ProductDelivery(_) => unavailable(),
        Settlement::ExecutionRequest(cause) => {
            crate::http::server::advancement_denial::advancement(cause)
        }
        Settlement::RelationalExecution(cause) => {
            crate::http::server::relational_execution_denial::denial(cause)
        }
        Settlement::SourceQueryInstallation(_) => unavailable(),
        Settlement::SourcePrincipal(_) => stale(),
        Settlement::SourceScope(kind) => entity(kind),
        Settlement::SourceQueryAdmission(kind) => admission(kind),
        Settlement::SourceQueryExecution(kind) => execution(kind),
        Settlement::RequestAuthorization(kind) => authorization_denial(kind),
        Settlement::Cancelled => cancelled(),
        Settlement::TimedOut => deadline(),
        Settlement::ProducerDomainDenied => malformed(),
        Settlement::Superseded
        | Settlement::PublicationStale
        | Settlement::ForeignSource
        | Settlement::ForeignDemand
        | Settlement::ForeignSettlement
        | Settlement::RetainedBasisUnavailable
        | Settlement::Closed => stale(),
        Settlement::WorkBudgetExceeded
        | Settlement::RetentionBudgetExceeded
        | Settlement::PublicationCapacityExceeded => exhausted(),
        Settlement::MissingApplicableProducer
        | Settlement::AmbiguousApplicableProducer
        | Settlement::ProducerUnavailable
        | Settlement::SchedulingRejected
        | Settlement::SchedulingDeferred
        | Settlement::NoEffect => unavailable(),
        Settlement::ProductSelection(kind) => product_selection(kind),
        Settlement::RequiredSuccessorStateUnavailable
        | Settlement::DuplicatePerformedSource
        | Settlement::IncompleteDependencyCoverage
        | Settlement::ManagedSourceExpansionUnsupported
        | Settlement::UnobservedWrite(_) => internal_denied(),
    }
}
