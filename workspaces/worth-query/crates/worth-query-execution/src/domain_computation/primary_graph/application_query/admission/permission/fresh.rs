//! One Query-owner preparation for the exact freshly resolved source access.

use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_declaration::facade::{
    application_query::{ApplicationQueryBinding, ApplicationQueryScopeBinding},
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;

use super::{PreparedApplicationQueryPermission, *};
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryControls;
use crate::domain_computation::primary_graph::{
    WorthQueryEntityResolutionDenial, WorthQueryObservedSource,
    WorthQueryPrincipalResolutionDenial, WorthQueryPrincipalResolutionMode,
};

type Parameters<Schema, Binding> =
    <<Binding as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value;
type ResultValue<Schema, Binding> =
    <<Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type Scope<Schema, Binding> =
    <<Binding as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<
        Schema,
    >>::Scope;

/// The terminal Query permission result retains original owner stops. A
/// caller cannot substitute a stale principal or scope into the read plan.
pub(in crate::domain_computation::primary_graph) enum FreshQueryPermissionStop<E> {
    Principal(WorthQueryPrincipalResolutionDenial),
    Scope(WorthQueryEntityResolutionDenial),
    Query(WorthQueryApplicationQueryAdmissionDenial),
    Inspect(E),
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Resolve the installed principal and original descriptive scope on this
    /// selected root, prepare the retained source, and consume permission in
    /// one scoped callback. Neither access identity can be supplied separately.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn with_fresh_retained_query_permission<
        'a,
        Binding,
        R,
        E,
    >(
        &'a self,
        installed: &'a WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
        validated_principal: Option<
            &worth_query_installation::facade::WorthQueryValidatedPrincipalBinding,
        >,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        observed: &WorthQueryObservedSource<Binding::Query>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        controls: WorthQueryApplicationQueryControls<'a, Schema>,
        admission: &mut InvalidationEditAdmission,
        inspect: impl for<'prepared> FnOnce(
            PreparedApplicationQueryPermission<
                'prepared,
                Schema,
                Binding::Query,
                Parameters<Schema, Binding>,
                ResultValue<Schema, Binding>,
                Binding::Principal,
                Binding::PrincipalIdentity,
                Scope<Schema, Binding>,
            >,
            &mut InvalidationEditAdmission,
        ) -> Result<R, E>,
    ) -> Result<R, FreshQueryPermissionStop<E>>
    where
        Binding: ApplicationQueryBinding<Schema>,
    {
        // The controls are the only request owner for this preparation. A
        // separate caller request could change principal/scope acceptance.
        let request = controls.request_scope();
        let principal = selected
            .resolve_authenticated_principal_issued(
                installed.principal_binding(),
                external,
                request,
                WorthQueryPrincipalResolutionMode::Ordinary,
                validated_principal,
                admission,
            )
            .map_err(FreshQueryPermissionStop::Principal)?;
        let scope = selected
            .resolve_retained_query_scope_issued(
                Binding::scope_field(),
                observed.retained_scope_selector(),
                request,
                admission,
            )
            .map_err(FreshQueryPermissionStop::Scope)?;
        let access =
            WorthQueryApplicationQueryAccessContext::new(principal.principal(), scope.scope());
        let query = installed.query();
        let source = self
            .prepare_application_query_from_observed(query, &access, observed, controls, admission)
            .map_err(FreshQueryPermissionStop::Query)?;
        let permission = self
            .prepare_application_query_permission(
                query, access, source, selected, request, &principal, &scope, admission,
            )
            .map_err(FreshQueryPermissionStop::Query)?;
        inspect(permission, admission).map_err(FreshQueryPermissionStop::Inspect)
    }
}
