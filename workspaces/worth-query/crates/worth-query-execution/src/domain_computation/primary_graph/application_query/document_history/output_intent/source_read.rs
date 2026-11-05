//! Admit the captured graph source at C before checking its exact restored D.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::{ApplicationQueryNativeAccess, WorthQueryApplicationQueryControls},
    WorthQueryApplicationOneShotResult, WorthQueryApplicationProjection,
    WorthQueryApplicationQueryAccessContext, WorthQueryEntityResolutionDenialKind as Resolution,
    WorthQueryEntityResolutionRefusalKind as Refusal, WorthQueryHistoryOutputAuthority,
    WorthQueryPrincipalResolutionMode,
};
use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;

type Scope<S, B> =
    <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope;
type Rows<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

impl WorthQueryHistoryOutputIntent {
    pub(in crate::domain_computation::primary_graph) fn execute_graph_source<S, B>(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        phase: &WorthQueryAdvancementPhase<'_>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        installed: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
    ) -> Result<
        (
            WorthQueryApplicationOneShotResult<B::Query, Rows<S, B>>,
            crate::basis::WorthQueryProductObservationLease,
        ),
        WorthQueryHistoryOutputReadDenial,
    >
    where
        S: ApplicationSchema,
        B: ApplicationQueryBinding<S>,
        Rows<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        use WorthQueryHistoryOutputReadDenial as Stop;
        let request = authority.request();
        authority
            .validate(phase, runtime, request)
            .map_err(Stop::Authority)?;
        if !authority.profile().uses_owned_model_graph()
            || !self.same_custody(authority.profile().source_intent())
            || !self.matches_binding(&self.backing.model, installed)
        {
            return Err(Stop::Authority(
                WorthQueryHistoryNavigationDenial::IntentDrift,
            ));
        }
        let execution = phase
            .checked_execution_request_for(&runtime.runtime.product_runtime)
            .map_err(|cause| Stop::Authority(WorthQueryHistoryNavigationDenial::Stopped(cause)))?;
        let current = runtime
            .on_branch(crate::basis::WorthQueryProductBranch::from_occurrence(
                authority.current().observation().lifecycle_incarnation(),
            ))
            .select_with_request(&execution)
            .map_err(|cause| {
                Stop::Authority(WorthQueryHistoryNavigationDenial::ProductAdmission(cause))
            })?;
        let control = current
            .prepare_shared_query_basis()
            .map_err(|(_, cause)| Stop::Resource(cause))?;
        let current = control.selected();
        if current.product().observation() != authority.current().observation() {
            return Err(Stop::Authority(
                WorthQueryHistoryNavigationDenial::HeadChanged,
            ));
        }
        let truth = runtime
            .on_branch(crate::basis::WorthQueryProductBranch::from_occurrence(
                authority
                    .original()
                    .product_observation()
                    .lifecycle_incarnation(),
            ))
            .select_with_request(&execution)
            .map_err(|cause| {
                Stop::Authority(WorthQueryHistoryNavigationDenial::ProductAdmission(cause))
            })?
            .prepare_shared_query_basis()
            .map_err(|(_, cause)| Stop::Resource(cause))?;
        // Earlier original trees may have published on this same private D.
        // Its actual head is data only; current C still supplies every grant.
        truth.selected().inspect_selected_program().map_err(|_| Stop::Authority(
            WorthQueryHistoryNavigationDenial::RequiresSemanticCorrection(
                crate::domain_computation::primary_graph::WorthQueryHistoryNavigationEligibilityDenial::ProgramUnavailable,
            ),
        ))?;
        let principal = current
            .resolve_authenticated_principal_issued(
                &execution,
                installed.principal_binding(),
                external,
                request,
                WorthQueryPrincipalResolutionMode::Ordinary,
                None,
            )
            .map_err(Stop::Principal)?;
        let current_scope = current.resolve_retained_query_scope_issued(
            &execution,
            B::scope_field(),
            &self.backing.scope,
            request,
        );
        let prepare = |scope: &crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<S, Scope<S, B>>,
                       issued: Option<&crate::domain_computation::primary_graph::WorthQueryIssuedSelectedScope<'_, '_, S, Scope<S, B>>>| {
            let access = WorthQueryApplicationQueryAccessContext::new(principal.principal(), scope);
            let (product, basis) = runtime.retain_selected_query_basis_admitted(&truth)
                .map_err(|cause| match cause {
                    crate::domain_computation::primary_graph::product_operation::SelectedQueryBasisRetentionStop::Basis => Stop::Authority(WorthQueryHistoryNavigationDenial::HeadChanged),
                    crate::domain_computation::primary_graph::product_operation::SelectedQueryBasisRetentionStop::Admission(cause) => Stop::Resource(cause),
                })?;
            let limits = runtime.resolve_application_query_limits(installed.limits());
            let controls = WorthQueryApplicationQueryControls::selected_read_one_shot_with_control(
                authority.current().retained_clone(), product, basis, limits.maximum_results(), request,
            );
            let query = installed.query();
            let (parameters, controls) = runtime.prepare_application_query_source_readmission(
                query, &access, &self.backing.parameters, controls,
            ).map_err(Stop::Admission)?;
            let mut native = ApplicationQueryNativeAccess::acquire();
            let permission = match issued {
                Some(issued) => runtime.prepare_application_query_permission_with_parameters(
                    &execution, query, access, query.identity(), parameters, controls,
                    current, Some(truth.selected()), request, &principal, issued, &mut native,
                ),
                None => runtime.prepare_historical_source_query_permission_with_parameters(
                    &execution, query, access, query.identity(), parameters, controls,
                    current, Some(truth.selected()), request, &principal, authority.producer_invocation(), &mut native,
                ),
            }.map_err(Stop::Admission)?;
            permission.require_history_principal(authority.producer_invocation()).map_err(Stop::Admission)?;
            let plan = runtime.finish_prepared_application_query_permission(&execution, permission).map_err(Stop::Admission)?;
            let reader = crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess::new(
                runtime, &truth, Some(&control), Some(authority.producer_invocation()), external, request,
            ).map_err(Stop::Demand)?;
            let result = reader.read_admitted_result::<B>(&execution, plan, &mut native)
                .map_err(|cause| Stop::Demand(crate::domain_computation::primary_graph::invariant_projection::WorthQueryOutputAvailabilityDenial::source_query_stop(cause).demand_stop()))?;
            Ok(result)
        };
        let result = match current_scope {
            Ok(scope) if scope.scope().entity_id() == self.backing.scope.subject() => prepare(scope.scope(), Some(&scope)),
            Ok(_) => self.backing.scope.with_description(&execution, |scope| prepare(scope, None)).map_err(|cause| Stop::Admission(super::super::super::admission::permission::fresh::historical_source::description_stop(cause, installed.query().name())))?,
            Err(cause) if cause.kind() == Resolution::Refused(Refusal::UnknownEntity) =>
                self.backing.scope.with_description(&execution, |scope| prepare(scope, None)).map_err(|cause| Stop::Admission(super::super::super::admission::permission::fresh::historical_source::description_stop(cause, installed.query().name())))?,
            Err(cause) => Err(Stop::Scope(cause)),
        };
        // Identity refusals also carry information about D. Confirm the same
        // C before delivering either an answer or a prepared refusal.
        authority
            .confirm_read(
                phase,
                runtime,
                request,
                result.map(|result| (result, truth.selected().product().read_lease())),
            )
            .map_err(Stop::Authority)?
    }
}
