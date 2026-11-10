//! Ordinary recording discovery keeps its actual action's ordinary C control.
use super::dependent::{DiscoveryBinding, QueryRows};
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::{
        output_source_observation::SourceQueryRequest, request_recipe::CapturedSourceQueryRequest,
        ApplicationQueryNativeAccess, SourceQueryReaderAccess,
    },
    product_operation::document_history::discovery_recipes::HistoryDiscoveryRecipeKey,
    WorthQueryApplicationOutputDemandSource, WorthQueryApplicationReadObservation,
    WorthQueryHistoryOutputReadDenial as Denial, WorthQueryHistoryRecordingOutputOrigin,
};
use worth_query_declaration::facade::application_query::ApplicationQueryScopeResolution;
impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    /// This is a real ordinary parent. Historical parent custody cannot enter
    /// this path, and a navigation cannot manufacture its recording origin.
    pub fn execute_recording_discovery<D, C>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        origin: &WorthQueryHistoryRecordingOutputOrigin,
        index: usize,
        parent: &WorthQuerySettledProgramOutput<S, P, D>,
        basis: &std::sync::Arc<WorthQueryApplicationReadObservation>,
        minimum: &WorthQueryApplicationReadObservation,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        intent: <ConnectionBinding<S, C> as WorthQueryApplicationDependentOutputConnection<S>>::Discovery,
    ) -> Result<
        WorthQueryApplicationOutputDemandSource<
            <DiscoveryBinding<S, C> as ApplicationQueryBinding<S>>::Query,
            QueryRows<S, DiscoveryBinding<S, C>>,
        >,
        Denial,
    >
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
        <DiscoveryBinding<S, C> as ApplicationQueryBinding<S>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                S,
                <DiscoveryBinding<S, C> as ApplicationQueryBinding<S>>::PrincipalIdentity,
            >,
        QueryRows<S, DiscoveryBinding<S, C>>: WorthQueryApplicationProjection<
            S,
            <DiscoveryBinding<S, C> as ApplicationQueryBinding<S>>::Query,
        >,
    {
        let runtime = self.runtime();
        let execution = phase.checked_execution_request_for(runtime.product_runtime())
            .map_err(|cause| Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::Stopped(cause)))?;
        let profile = origin
            .discovery_profile(index)
            .filter(|profile| profile.matches_program(self))
            .ok_or_else(foreign)?;
        if basis.branch_incarnation() != origin.candidate().occurrence()
            || basis.selected_commit().ordinal() < origin.observation().selected_commit().ordinal()
        {
            return Err(foreign());
        }
        let key = HistoryDiscoveryRecipeKey::new::<C>(
            profile,
            parent.retained.source_route().ok_or_else(foreign)?,
        );
        let recipe = origin.recipe_recorder().record(&execution, key, || {
            CapturedSourceQueryRequest::capture::<S, DiscoveryBinding<S, C>>(
                &execution,
                runtime,
                origin.recipe_step(),
                intent,
            )
        })?;
        let selected = runtime.select_application_read_observation_with_request(&execution, basis)
            .map_err(|cause| Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::ProductAdmission(cause)))?
            .prepare_shared_query_basis().map_err(|(_, cause)| Denial::Resource(cause))?;
        let reader =
            SourceQueryReaderAccess::new(runtime, &selected, None, None, external, request)
                .map_err(Denial::Demand)?;
        reader.with_request_plan::<DiscoveryBinding<S, C>, _, Denial>(
            &execution,
            SourceQueryRequest::Recorded(recipe),
            &std::cell::Cell::default(),
            &mut ApplicationQueryNativeAccess::acquire(),
            |plan, native| {
                self.validate_dependent_output_parent::<D, C>(parent)
                    .map_err(Denial::Demand)?;
                self.select_dependent_output_basis(&execution, parent, basis, minimum)
                    .map_err(Denial::Demand)?;
                reader
                    .read_admitted_result::<DiscoveryBinding<S, C>>(&execution, plan, native)
                    .map(|result| result.into_admitted_disclosed().into_output_demand_source())
                    .map_err(Denial::from)
            },
        )
    }
}
fn foreign() -> Denial {
    Denial::Demand(WorthQueryOutputDemandDenial::new(
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSource,
        "ordinary recording discovery",
    ))
}
