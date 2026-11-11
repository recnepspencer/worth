//! Missing current sources retain C authority and qualify identity at navigation D.
use super::*;
use crate::domain_computation::primary_graph::{
    product_operation::document_history::HistoryProducerInvocation,
    WorthQueryApplicationEntityIdentity, WorthQueryEntityResolutionDenialKind as Resolution,
    WorthQueryEntityResolutionRefusalKind as Refusal,
};

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn with_historical_source_permission<'a, B, R, E>(
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &'a WorthQueryInstalledApplicationQueryBinding<S, B>,
        validated_principal: Option<
            &worth_query_installation::facade::WorthQueryValidatedPrincipalBinding,
        >,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        observed: &WorthQueryObservedSource<B::Query>,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
        truth: Option<&WorthQuerySelectedProductOperation<'_, S>>,
        controls: WorthQueryApplicationQueryControls<'a, S>,
        history: HistoryProducerInvocation<'_>,
        inspect: impl for<'prepared> FnOnce(
            PreparedApplicationQueryPermission<
                'prepared,
                S,
                B::Query,
                Parameters<S, B>,
                ResultValue<S, B>,
                B::Principal,
                B::PrincipalIdentity,
                Scope<S, B>,
            >,
        ) -> Result<R, E>,
    ) -> Result<R, FreshQueryPermissionStop<E>>
    where
        B: ApplicationQueryBinding<S>,
    {
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
        let current_scope = selected.resolve_retained_query_scope_issued(
            execution,
            B::scope_field(),
            observed.retained_scope_selector(),
            request,
        );
        let prepare = |scope: &WorthQueryApplicationEntityIdentity<S, Scope<S, B>>,
                       issued: Option<
            &crate::domain_computation::primary_graph::WorthQueryIssuedSelectedScope<
                '_,
                '_,
                S,
                Scope<S, B>,
            >,
        >| {
            let access = WorthQueryApplicationQueryAccessContext::new(principal.principal(), scope);
            // This validates the installed binding, genuine original selector,
            // request, principal and parameter capabilities at their owners.
            let source = self
                .prepare_application_query_from_observed(
                    installed.query(),
                    &access,
                    observed,
                    controls,
                )
                .map_err(FreshQueryPermissionStop::Query)?;
            let permission = match issued {
                Some(issued) => self.prepare_application_query_permission(
                    execution, installed.query(), access, source, selected, truth,
                    request, &principal, issued,
                ),
                None => {
                    let (identity, parameters, controls) = source.into_permission_parts();
                    self.prepare_historical_source_query_permission_with_parameters(
                        execution, installed.query(), access, &identity, parameters,
                        controls, selected, truth, request, &principal, history,
                        &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess::acquire(),
                    )
                }
            }.map_err(FreshQueryPermissionStop::Query)?;
            inspect(permission).map_err(FreshQueryPermissionStop::Inspect)
        };
        match current_scope {
            Ok(scope)
                if scope.scope().entity_id() == observed.retained_scope_selector().subject() =>
            {
                prepare(scope.scope(), Some(&scope))
            }
            Ok(_) => observed
                .retained_scope_selector()
                .with_description(execution, |description| prepare(description, None))
                .map_err(|stop| {
                    FreshQueryPermissionStop::Query(description_stop(
                        stop,
                        installed.query().name(),
                    ))
                })?,
            Err(cause) if cause.kind() == Resolution::Refused(Refusal::UnknownEntity) => observed
                .retained_scope_selector()
                .with_description(execution, |description| prepare(description, None))
                .map_err(|stop| {
                    FreshQueryPermissionStop::Query(description_stop(
                        stop,
                        installed.query().name(),
                    ))
                })?,
            Err(cause) => Err(FreshQueryPermissionStop::Scope(cause)),
        }
    }
}

pub(in crate::domain_computation::primary_graph::application_query) fn description_stop(
    stop: Result<
        crate::domain_computation::WorthQueryProviderSessionDenialKind,
        crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    >,
    subject: &str,
) -> WorthQueryApplicationQueryAdmissionDenial {
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Stop;
    denial(
        match stop {
            Ok(kind) => WorthQueryApplicationQueryAdmissionDenialKind::ExecutionRequest(kind),
            Err(Stop::Cancelled) => WorthQueryApplicationQueryAdmissionDenialKind::Cancelled,
            Err(Stop::TimedOut) => WorthQueryApplicationQueryAdmissionDenialKind::DeadlineExceeded,
        },
        subject,
    )
}
