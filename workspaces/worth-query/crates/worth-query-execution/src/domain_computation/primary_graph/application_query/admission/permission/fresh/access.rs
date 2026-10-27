//! Fresh issued identities stay borrowed until the retained-source continuation returns.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryIssuedSelectedPrincipal, WorthQueryIssuedSelectedScope,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Ordinary and governed reads share actual principal/scope resolution and
    /// original source preparation. No caller can replace these issued guards.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn with_fresh_retained_query_access<
        'a,
        'selected,
        'runtime,
        Binding,
        R,
        E,
    >(
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &'a WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
        validated_principal: Option<
            &worth_query_installation::facade::WorthQueryValidatedPrincipalBinding,
        >,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        observed: &WorthQueryObservedSource<Binding::Query>,
        selected: &'selected WorthQuerySelectedProductOperation<'runtime, Schema>,
        controls: WorthQueryApplicationQueryControls<'a, Schema>,
        inspect: impl for<'fresh> FnOnce(
            WorthQueryApplicationQueryAccessContext<
                'fresh,
                Schema,
                Binding::Principal,
                Binding::PrincipalIdentity,
                Scope<Schema, Binding>,
            >,
            PreparedApplicationQuerySourceReadmission<'fresh, Schema>,
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
        // Controls retain the original request; a separate request cannot
        // change which selected principal or scope is accepted here.
        let request = controls.request_scope();
        let principal = selected
            .resolve_authenticated_principal_issued(
                execution,
                installed.principal_binding(),
                external,
                request,
                WorthQueryPrincipalResolutionMode::Ordinary,
                validated_principal,
            )
            .map_err(FreshQueryPermissionStop::Principal)?;
        let scope = selected
            .resolve_retained_query_scope_issued(
                execution,
                Binding::scope_field(),
                observed.retained_scope_selector(),
                request,
            )
            .map_err(FreshQueryPermissionStop::Scope)?;
        let access =
            WorthQueryApplicationQueryAccessContext::new(principal.principal(), scope.scope());
        let source = self
            .prepare_application_query_from_observed(installed.query(), &access, observed, controls)
            .map_err(FreshQueryPermissionStop::Query)?;
        inspect(access, source, &principal, &scope)
    }
}
