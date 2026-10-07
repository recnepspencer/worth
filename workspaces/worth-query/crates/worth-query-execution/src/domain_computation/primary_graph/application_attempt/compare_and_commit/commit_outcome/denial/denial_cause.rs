//! Typed owner evidence carried without changing the denial category.

#[derive(Debug)]
pub(super) enum DenialCause {
    CustomInvariant(crate::domain_computation::WorthQueryCustomInvariantDenial),
    /// The request's own authorization stopped the commit: its security
    /// basis on the branch, or what it may do there.
    RequestAuthority(
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind,
    ),
    Execution(
        Box<
            Result<
                crate::domain_computation::WorthQueryProviderSessionDenialKind,
                crate::domain_computation::WorthQueryProviderSessionControlStopKind,
            >,
        >,
    ),
}
