//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn with_fresh_retained_query_identities_for_test<'a, 'selected, 'runtime, Binding, R, E>(
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &'a WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
        validated_principal: Option<
            &worth_query_installation::facade::WorthQueryValidatedPrincipalBinding,
        >,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        observed: &WorthQueryObservedSource<impl Sized>,
        selected: &'selected WorthQuerySelectedProductOperation<'runtime, Schema>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        inspect: impl for<'fresh> FnOnce(
            WorthQueryApplicationQueryAccessContext<
                'fresh,
                Schema,
                Binding::Principal,
                Binding::PrincipalIdentity,
                Scope<Schema, Binding>,
            >,
            &'fresh WorthQueryIssuedSelectedPrincipal<
                'selected,
                'runtime,
                Schema,
                Binding::Principal,
                Binding::PrincipalIdentity,
            >,
            &'fresh WorthQueryIssuedSelectedScope<
                'selected,
                'runtime,
                Schema,
                Scope<Schema, Binding>,
            >,
        ) -> Result<R, FreshQueryPermissionStop<E>>,
    ) -> Result<R, FreshQueryPermissionStop<E>>
    where
        Binding: ApplicationQueryBinding<Schema>,
    {
        Self::with_fresh_retained_query_identities(
            self,
            execution,
            installed,
            validated_principal,
            external,
            observed,
            selected,
            request,
            inspect,
        )
    }
}
