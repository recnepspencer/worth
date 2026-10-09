//! Typed owner evidence carried without changing the denial category.

#[derive(Debug)]
pub(super) enum DenialCause {
    SourceRebase(crate::domain_computation::primary_graph::provider::PreparedRebaseDenial),
    InvariantExecution(crate::domain_computation::WorthQueryInvariantExecutionFailure),
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
