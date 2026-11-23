//! Typed owner evidence carried without changing the denial category.

#[derive(Debug)]
pub enum DenialCause {
    CoupledProducer(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial),
    DomainValue(worth_query_installation::facade::ApplicationValueDecodeDenial),
    DomainValueAdmission(crate::domain_computation::primary_graph::WorthQueryValueDecodeDenial),
    UnobservedWrite(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationUnobservedWrite),
    SigningLifecycleAttempt(
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    ),
    SourceRebase(crate::domain_computation::primary_graph::provider::PreparedRebaseDenial),
    InvariantExecution(crate::domain_computation::WorthQueryInvariantExecutionFailure),
    /// The operation's declared output names were refused against the
    /// commit basis before the transaction was staged.
    PersistentNameOutput(worth_query_execution_naming::facade::WorthQueryOutputNameDenial),
    ProviderSession(crate::domain_computation::WorthQueryProviderSessionFailure),
    DecisionReadSet(crate::domain_computation::WorthQueryDecisionReadSetFailure),
    /// The request's own authorization stopped the commit: its security
    /// basis on the branch, or what it may do there.
    RequestAuthority(
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind,
    ),
    Execution(
        Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    ),
}
