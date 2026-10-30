//! Inverse entity postings retain each shared cell's actual original grant.
use super::*;
use crate::domain_computation::primary_graph::application_query::derived_view::ViewDependency;
use worth_execution::{ExecutionOrderedMap, ExecutionOrderedMapDenial};

#[derive(Clone, Default)]
pub(super) struct OccurrencePostings {
    entities: ExecutionOrderedMap<EntityId, ExecutionOrderedMap<EntityId, ()>>,
}
impl OccurrencePostings {
    pub(super) fn entries_for_entity(&self, entity: EntityId) -> impl Iterator<Item = &EntityId> {
        self.entities
            .get(&entity)
            .into_iter()
            .flat_map(|entries| entries.keys())
    }

    pub(super) fn insert_entry(
        &mut self,
        entry: EntityId,
        dependencies: &BTreeSet<ViewDependency>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), MaterialDenial> {
        for dependency in dependencies {
            let entity = dependency.entity();
            let mut entries = self.entities.get(&entity).cloned().unwrap_or_default();
            if entries.contains_key(&entry) {
                continue;
            }
            entries
                .insert_cloned_native_for_request(&entry, &(), *execution)
                .map_err(|cause| ordered_denial(execution, cause))?;
            self.entities
                .insert_cloned_native_for_request(&entity, &entries, *execution)
                .map_err(|cause| ordered_denial(execution, cause))?;
        }
        Ok(())
    }

    pub(super) fn remove_entry(
        &mut self,
        entry: EntityId,
        dependencies: &BTreeSet<ViewDependency>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), MaterialDenial> {
        for dependency in dependencies {
            let entity = dependency.entity();
            let Some(mut entries) = self.entities.get(&entity).cloned() else {
                continue;
            };
            if !entries
                .remove_for_request(&entry, *execution)
                .map_err(|cause| ordered_denial(execution, cause))?
            {
                continue;
            }
            if entries.is_empty() {
                self.entities
                    .remove_for_request(&entity, *execution)
                    .map_err(|cause| ordered_denial(execution, cause))?;
            } else {
                self.entities
                    .insert_cloned_native_for_request(&entity, &entries, *execution)
                    .map_err(|cause| ordered_denial(execution, cause))?;
            }
        }
        Ok(())
    }
}

pub(super) fn ordered_denial(
    execution: &ExecutionRequest<'_, '_>,
    cause: ExecutionOrderedMapDenial,
) -> MaterialDenial {
    match cause {
        ExecutionOrderedMapDenial::Request(cause) => MaterialDenial::Storage(cause.into()),
        ExecutionOrderedMapDenial::CapacityOverflow => {
            MaterialDenial::Storage(RelationalExecutionDenialCause::ChargedBytesOverflow)
        }
        ExecutionOrderedMapDenial::Allocation(cause) => {
            MaterialDenial::Native(super::super::service::retain_failure(execution, cause))
        }
    }
}
