//! The ordinary currentness owner also verifies untouched restored routes.
use super::*;
impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub(in crate::domain_computation::primary_graph) fn selected_route_is_current(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        route: &worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let runtime = self.application();
        let graph = &runtime.runtime.primary_provider.graph;
        let read = lineage_read(|| {
            graph
                .output_lineage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .source_facts_for_selected_route(
                    execution,
                    runtime.runtime.authority_identity().as_u64(),
                    &runtime.installed_schema.binding_identity(),
                    self.product().observation(),
                    route,
                )
        })?;
        let Some(read) = read else {
            return Ok(false);
        };
        let Some(publication) = read.required_publication.as_ref() else {
            return Ok(false);
        };
        if !runtime
            .installed_producers
            .implementation_matches(publication.producer(), route.implementation)
        {
            return Ok(false);
        }
        // The existing owner checks actual current storage, independent roster
        // and presence, original source facts, the output witness and upstreams.
        match self.require_current_read(execution, native, &read, None) {
            Ok(()) => Ok(true),
            Err(cause) if matches!(cause.kind(), WorthQueryOutputDemandDenialKind::Superseded) => {
                Ok(false)
            }
            Err(cause) => Err(cause),
        }
    }
}
