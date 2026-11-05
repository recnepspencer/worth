//! Every restored Native route must have an actual freshly qualified family.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryHistoryOutputQualificationDenial as Denial, WorthQueryOutputDemandSettlement,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;

pub(super) fn require<'s, S: ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
    settlements: impl Iterator<Item = &'s WorthQueryOutputDemandSettlement> + Clone,
) -> Result<(), Denial> {
    let Some(selected) = authority.selected_graph() else {
        return Ok(());
    };
    let graph = &runtime.runtime.primary_provider.graph;
    graph.with_runtime(execution, |native| {
        let evidence = native.read_truth().read_historical_selection(execution, selected,
            &authority.original().product_observation().basis().relational_basis().observation())
            .map_err(Denial::SelectionRead)?;
        for route in evidence.selected_output_routes(execution).map_err(Denial::SelectionRead)? {
            let route = route.map_err(Denial::SelectionRead)?;
            let mut covered = false;
            for settlement in settlements.clone() {
                let work = route.family.len().max(settlement.output_family_identity().len()) as u64 + 8;
                execution.run_native_with_checkpoint(|context| {
                    crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, work)
                }).map_err(|cause| Denial::SelectionRead(worth_relational::facade::runtime::RelationalHistoricalSelectionReadDenial::Request(cause)))?
                    .0.map_err(|stop| Denial::SelectionRead(worth_relational::facade::runtime::RelationalHistoricalSelectionReadDenial::Request(worth_execution::WorkCeilingDenial::Stopped(stop))))?;
                if settlement.covers_historical_route(route) { covered = true; break; }
            }
            if !covered { return Err(Denial::Incomplete { model: evidence.model() }); }
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
            require(&execution, self.runtime(), authority, std::iter::once(root).chain(dependents.clone()))?;
            self.qualify_original_history_outputs::<R>(phase, access, authority.request(), source, root, dependents, dependencies).map_err(Denial::Original)
        })();
        authority.confirm_read(phase, self.runtime(), authority.request(), result).map_err(control)?
    }
}
