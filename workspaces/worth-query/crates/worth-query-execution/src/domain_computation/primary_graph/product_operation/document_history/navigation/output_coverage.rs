//! Every restored Native route must have an actual freshly qualified family.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryHistoryOutputQualificationDenial as Denial, WorthQueryOutputDemandSettlement,
};
use worth_query_installation::facade::ApplicationSchema;

pub(super) fn require<'s, S: ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    selected_output: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
        '_,
        S,
    >,
    authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
    settlements: impl Iterator<Item = &'s WorthQueryOutputDemandSettlement> + Clone,
) -> Result<(), Denial> {
    if authority.selected_graph().is_none() {
        return Ok(());
    }
    let graph = &selected_output.application().runtime.primary_provider.graph;
    graph.with_runtime(execution, |native| {
        let evidence = authority.selected_output_read(execution, native)
            .map_err(Denial::SelectionRead)?;
        for route in evidence.selected_output_routes(execution).map_err(Denial::SelectionRead)? {
            let route = route.map_err(Denial::SelectionRead)?;
            let mut covered = false;
            for settlement in settlements.clone() {
                let work = route.family.len().max(settlement.output_family_identity().len()) as u64 + 8;
                charge(execution, work)?;
                if settlement.covers_historical_route(route) { covered = true; break; }
            }
            if !covered && !selected_output.selected_route_is_current(execution, native, route)
                .map_err(|cause| Denial::Original(crate::domain_computation::primary_graph::WorthQueryOriginalReadOutputDenial::Output(cause)))?
            { return Err(Denial::Incomplete { model: evidence.model() }); }
        }
        Ok(())
    })
}

impl<S: ApplicationSchema + 'static, P: worth_query_declaration::facade::application_program::ApplicationProgramDefinition<S>>
    crate::domain_computation::primary_graph::application_installation::WorthQueryProgramApplicationRuntime<S, P>
{
    /// The graph path adds Native route completeness before using the same
    /// original-tree qualifier. ScopeEntityRecord retains its existing entrance.
    pub fn qualify_original_graph_history_outputs<'s, R: worth_query_declaration::facade::application_program::ApplicationOutputGraphShape<S>>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        source: &WorthQueryHistoryOutputSource,
        root: &'s WorthQueryOutputDemandSettlement,
        dependents: impl Iterator<Item = &'s WorthQueryOutputDemandSettlement> + Clone,
        dependencies: Option<&crate::publication_boundary::discovery_currentness::WorthQueryRetainedQueryDependencies>,
    ) -> Result<WorthQueryCompletedHistoryOutputs, Denial> {
        let control = |cause| Denial::Control(Box::new(cause));
        authority.validate(phase, self.runtime(), authority.request()).map_err(control)?;
        let result = (|| {
            if !source.profile.uses_owned_model_graph()
                || !source.profile.same_custody(authority.profile())
                || !Arc::ptr_eq(&source.source, authority.original())
            {
                return Err(control(WorthQueryHistoryNavigationDenial::IntentDrift));
            }
            let execution = phase.checked_execution_request_for(&self.runtime().runtime.product_runtime)
                .map_err(|cause| control(WorthQueryHistoryNavigationDenial::Stopped(cause)))?;
            let completed = self.qualify_original_history_outputs::<R>(phase, access, authority.request(), source, root, dependents.clone(), dependencies).map_err(Denial::Original)?;
            let selected_output = self.runtime().select_application_read_observation(phase, completed.original.endpoint())
                .map_err(|cause| Denial::Original(crate::domain_computation::primary_graph::WorthQueryOriginalReadOutputDenial::Observation(cause)))?;
            require(&execution, &selected_output, authority, std::iter::once(root).chain(dependents))?;
            Ok(completed)
        })();
        authority.confirm_read(phase, self.runtime(), authority.request(), result).map_err(control)?
    }
}
