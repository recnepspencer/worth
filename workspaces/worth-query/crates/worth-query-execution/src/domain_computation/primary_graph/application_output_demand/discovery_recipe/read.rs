//! Re-enter a recorded discovery with the ordinary installed read owners.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationOutputDemandSource,
    WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationReadObservation, WorthQueryPrincipalResolutionMode,
    WorthQueryProductQueryControls,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn read_recorded_output_discovery<B>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        recipe: &WorthQueryOutputDiscoveryRecipe<B::Input>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        original: &WorthQueryApplicationReadObservation,
    ) -> Result<
        WorthQueryApplicationOutputDemandSource<
            B::Query,
            <B::ResultBinding as ApplicationStructuredValueBinding>::Value,
        >,
        Denial,
    >
    where
        B: ApplicationQueryBinding<S>,
        B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
        <B::ResultBinding as ApplicationStructuredValueBinding>::Value:
            WorthQueryApplicationProjection<S, B::Query>,
    {
        let Original::Recorded(data) = &recipe.original else {
            return Err(unavailable());
        };
        let execution = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|cause| {
                Denial::new(Kind::ExecutionRequest(cause.into()), "recorded discovery")
            })?;
        let binding = self
            .installed_schema()
            .installed_query_binding::<B>()
            .map_err(|_| unavailable())?;
        let query = binding.query();
        let field = B::scope_field();
        if query.identity().as_bytes() != &data.query
            || data.field
                != (
                    field.entity().to_owned(),
                    field.aspect().to_owned(),
                    field.field().to_owned(),
                )
        {
            return Err(unavailable());
        }
        let retained = self
            .select_application_read_observation_with_request(&execution, original)
            .map_err(|cause| Denial::product_selection(cause, "recorded discovery"))?;
        let current = self
            .on_branch(crate::basis::WorthQueryProductBranch::from_occurrence(
                original.branch_incarnation(),
            ))
            .select_with_request(&execution)
            .map_err(|cause| Denial::product_selection(cause, "recorded discovery"))?;
        let principal = current
            .resolve_authenticated_principal_admitted(
                &execution,
                binding.principal_binding(),
                external,
                request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(|cause| {
                Denial::new(
                    Kind::of_principal_resolution(cause.kind()),
                    format!("recorded discovery: {cause:?}"),
                )
            })?;
        let value = match &data.scope {
            Some(value) => value.clone(),
            None => {
                let value =
                    B::ScopeBinding::request_principal_value(principal.principal_identity())
                        .ok_or_else(unavailable)?;
                <<B::ScopeBinding as ApplicationQueryScopeBinding<S>>::Field as RecordSelection>::Binding::encode(&value).map_err(|_| unavailable())?
            }
        };
        let scope = current.resolve_query_scope_value_with_native_access(&execution,field,None,&value,request,&mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess::acquire()).map_err(|cause| Denial::new(Kind::of_scope_resolution(cause.kind()),format!("recorded discovery: {cause:?}")))?;
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
        let parameters = worth_query_admission::facade::application_query::readmit_recorded_application_query_parameters(query,data.parameters.clone()).map_err(|_| unavailable())?;
        let (_, controls) = current
            .retained_query_controls(
                &execution,
                retained,
                query.name(),
                WorthQueryProductQueryControls::new(
                    self.resolve_application_query_limits(binding.limits())
                        .maximum_results(),
                    request,
                ),
            )
            .map_err(|_| unavailable())?;
        let plan = self
            .finish_application_query_readmission(
                &execution, query, &access, parameters, controls, None,
            )
            .map_err(|cause| {
                Denial::new(
                    Kind::SourceQueryAdmission(cause.kind()),
                    "recorded discovery admission",
                )
            })?;
        self.execute_application_query_one_shot_with_request(&execution, plan)
            .map(|read| read.into_admitted_disclosed().into_output_demand_source())
            .map_err(|cause| {
                Denial::new(
                    Kind::SourceQueryExecution(cause.kind()),
                    "recorded discovery execution",
                )
            })
    }
}
