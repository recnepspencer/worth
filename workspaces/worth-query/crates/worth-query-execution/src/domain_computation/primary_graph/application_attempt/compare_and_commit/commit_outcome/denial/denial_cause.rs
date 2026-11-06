//! Typed owner evidence carried without changing the denial category.

#[derive(Debug)]
pub(super) enum DenialCause {
    DomainValue(worth_query_installation::facade::ApplicationValueDecodeDenial),
    DomainValueAdmission(crate::domain_computation::vocabulary::primary_graph::value_decode::WorthQueryValueDecodeDenial),
    UnobservedWrite(crate::domain_computation::primary_graph::vocabulary::application_attempt::unobserved_write::WorthQueryApplicationUnobservedWrite),
    SigningLifecycleAttempt(
        crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial,
    ),
    SourceRebase(crate::domain_computation::primary_graph::provider::PreparedRebaseDenial),
    InvariantExecution(crate::domain_computation::WorthQueryInvariantExecutionFailure),
    ProviderSession(crate::domain_computation::WorthQueryProviderSessionFailure),
    DecisionReadSet(crate::domain_computation::WorthQueryDecisionReadSetFailure),
    /// The request's own authorization stopped the commit: its security
    /// basis on the branch, or what it may do there.
    RequestAuthority(
        crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind,
    ),
    Execution(
        Result<
            crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind,
        >,
    ),
}
