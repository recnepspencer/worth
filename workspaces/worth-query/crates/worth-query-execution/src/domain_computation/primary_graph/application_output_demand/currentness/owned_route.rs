//! A Native current roster addresses the same lineage/currentness verifier.
use super::*;
use worth_relational::facade::{
    branch::RelationalModelOwnershipDenial, identity::EntityId, runtime::RelationalRuntime,
};
pub(in crate::domain_computation::primary_graph) enum CurrentOwnedRouteDenial {
    Ownership(RelationalModelOwnershipDenial),
    Output(WorthQueryOutputDemandDenial),
}
impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S> {
    pub(in crate::domain_computation::primary_graph) fn current_owned_route_is_current(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
        model: EntityId,
        source: EntityId,
        family: &str,
        partition: [u8; 32],
    ) -> Result<bool, CurrentOwnedRouteDenial> {
        let graph = &self.application().runtime.primary_provider.graph;
        let Some(publisher) = graph
            .application_publication
            .as_ref()
            .filter(|publisher| publisher.native.owns_source_scopes())
        else {
            return Ok(false);
        };
        let roster = native
            .observe_owned_model_publications(
                execution,
                &publisher.native,
                self.product().relational_basis(),
                graph.layout.installed_history_ownership(),
                model,
            )
            .map_err(CurrentOwnedRouteDenial::Ownership)?;
        for route in roster.routes() {
            let worth_relational::facade::branch::RelationalOwnedModelOutputRoute::Published(route) =
                route
            else {
                continue;
            };
            let meaning = route.meaning();
            execution
                .run_native_with_checkpoint(|context| {
                    crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                        context,
                        (meaning.family.len() + family.len() + 65) as u64,
                    )
                })
                .map_err(|cause| {
                    CurrentOwnedRouteDenial::Output(
                        crate::domain_computation::primary_graph::output_reuse::read_denial(
                            cause.into(),
                        ),
                    )
                })?
                .0
                .map_err(|stop| {
                    CurrentOwnedRouteDenial::Output(
                        crate::domain_computation::primary_graph::output_reuse::read_denial(
                            worth_execution::WorkCeilingDenial::Stopped(stop).into(),
                        ),
                    )
                })?;
            if meaning.family == family
                && meaning.source_scope == source
                && meaning.source_partition == partition
            {
                return self
                    .current_route_is_current(execution, native, &route)
                    .map_err(CurrentOwnedRouteDenial::Output);
            }
        }
        Ok(false)
    }
}
