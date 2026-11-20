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

    pub(super) fn check_complete(
        &self,
        scope: &ManagedIndexSourceScope<'_>,
        entries: &ExecutionOrderedMap<EntityId, Arc<Entry>>,
        changed_keys: &BTreeSet<EntityId>,
    ) -> Result<(), MaterialDenial> {
        let budget = scope.read_budget();
        for row in scope.changed_entities() {
            budget
                .checkpoint(1)
                .map_err(DerivedRelationReadDenial::Structural)?;
            if entries.contains_key(&row) && !changed_keys.contains(&row) {
                return Err(ManagedIndexSourcePreparationDenial::IncompletePreparation.into());
            }
        }
        let changed_records = scope.changed_record_entities();
        if changed_records.len() <= self.entities.len() {
            for entity in changed_records {
                budget
                    .checkpoint(1)
                    .map_err(DerivedRelationReadDenial::Structural)?;
                self.check_entity_dependents(entity, changed_keys, &budget)?;
            }
        } else {
            for &entity in self.entities.keys() {
                budget
                    .checkpoint(1)
                    .map_err(DerivedRelationReadDenial::Structural)?;
                if scope.is_changed_record_entity(entity) {
                    self.check_entity_dependents(entity, changed_keys, &budget)?;
                }
            }
        }
        Ok(())
    }

    fn check_entity_dependents(
        &self,
        entity: EntityId,
        changed_keys: &BTreeSet<EntityId>,
        budget: &worth_relational::facade::runtime::CustomInvariantReadBudget<'_>,
    ) -> Result<(), MaterialDenial> {
        for key in self.entries_for_entity(entity) {
            budget
                .checkpoint(1)
                .map_err(DerivedRelationReadDenial::Structural)?;
            if !changed_keys.contains(key) {
                return Err(ManagedIndexSourcePreparationDenial::IncompletePreparation.into());
            }
        }
        Ok(())
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
            MaterialDenial::Native(super::super::failure::retain_failure(execution, cause))
        }
    }
}
