//! Native selected-route addresses enter the existing canonical lineage reader.
use super::*;
use worth_relational::facade::mvcc::{
    CompanionPreflightStop as Stop, RequiredOutputPublicationMeaning,
};

impl WorthQueryApplicationOutputLineage {
    /// A direct restored read accepts only the transported row at this exact
    /// selected version. It is not a demand, a settlement or producer authority.
    pub(in crate::domain_computation::primary_graph) fn source_facts_for_selected_route(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        route: &RequiredOutputPublicationMeaning,
    ) -> Result<Option<RetainedOutputCurrentnessRead>, Stop> {
        let Some(bindings) = self.output_families.get(route.family.as_str()) else {
            return Ok(None);
        };
        let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(route.source_scope);
        let mut found = None;
        let mut matched = false;
        for (binding, _) in bindings {
            for candidate in self.retained_output_candidates_metered(
                runtime_authority,
                schema,
                scope,
                observation.lifecycle_incarnation(),
                observation.reference_generation().get(),
                std::slice::from_ref(binding),
                route.source_partition,
                execution,
            )? {
                if !candidate
                    .required_publication
                    .as_ref()
                    .is_some_and(|publication| {
                        publication.is_historical_selection() && publication.meaning() == route
                    })
                {
                    continue;
                }
                if matched {
                    return Err(Stop::SelectedSourceMismatch);
                }
                matched = true;
                let pin = &candidate.settlement_identity;
                let coordinate = pin.coordinate();
                let row = self.recorded_at_partition_slot(
                    pin.source(),
                    coordinate.occurrence,
                    coordinate.generation,
                    Some(route.source_partition),
                    pin.slot(),
                );
                if !self.implementation_is_current(row) {
                    return Ok(None);
                }
                found = retained_currentness_read(row)?;
            }
        }
        Ok(found)
    }
}
