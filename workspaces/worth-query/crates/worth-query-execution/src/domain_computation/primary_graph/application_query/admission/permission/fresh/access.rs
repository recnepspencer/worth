//! Fresh issued identities stay borrowed until the retained-source continuation returns.
use super::*;
use crate::domain_computation::primary_graph::application_query::SourceReadmissionScopedOps;
use crate::domain_computation::primary_graph::{
    WorthQueryIssuedSelectedPrincipal, WorthQueryIssuedSelectedScope,
};

pub(in crate::domain_computation::primary_graph) trait AccessScopedOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    #[allow(clippy::too_many_arguments)]
    fn with_fresh_retained_query_access<'a, 'selected, 'runtime, Binding, R, E>(
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
        let request = controls.request_scope();
        self.with_fresh_retained_query_identities(
            execution,
            installed,
            validated_principal,
            external,
            observed,
            selected,
            request,
            |access, principal, scope| {
                let source = self
                    .prepare_application_query_from_observed(
                        installed.query(),
                        &access,
                        observed,
                        controls,
                    )
                    .map_err(FreshQueryPermissionStop::Query)?;
                inspect(access, source, principal, scope)
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn with_fresh_retained_query_identities<'a, 'selected, 'runtime, Binding, R, E>(
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
        self.with_fresh_inspection_scope_identities(
            execution,
            installed,
            validated_principal,
            external,
            observed.retained_scope_selector(),
            selected,
            request,
            inspect,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn with_fresh_inspection_scope_identities<'a, 'selected, 'runtime, Binding, R, E>(
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &'a WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
        validated_principal: Option<
            &worth_query_installation::facade::WorthQueryValidatedPrincipalBinding,
        >,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        selector: &crate::domain_computation::vocabulary::primary_graph::retained_selector::WorthQueryObservedScopeSelector,
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
                selector,
                request,
            )
            .map_err(FreshQueryPermissionStop::Scope)?;
        let access =
            WorthQueryApplicationQueryAccessContext::new(principal.principal(), scope.scope());
        inspect(access, &principal, &scope)
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > AccessScopedOps<Schema> for HostRuntime
{
}

pub(crate) mod court_doors_for_test;
pub use court_doors_for_test::AccessDoorsOps;
use crate::domain_computation::vocabulary::primary_graph::resolution::WorthQueryPrincipalResolutionMode;
