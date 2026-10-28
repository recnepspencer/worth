//! Collect each actual physical source once, preserving canonical key order.
use super::*;
use crate::domain_computation::primary_graph::{
    structural_walk::StructuralWalkObservation, WorthQueryPrimaryGraphLayout,
};
use std::collections::{BTreeMap, BTreeSet};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, CanonicalFieldPath, FieldKey, LocatorAuthority,
};
use worth_relational::facade::{
    identity::KindId,
    runtime::{
        AdjacencyStructuralRevisionDenial, RelationalAdjacencyDirection, VisibilityProjectionView,
    },
    transactions::RelationalExecutionDenialCause as Cause,
};

mod allocation;

#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct SourceFactsBuilder {
    entities: BTreeSet<EntityId>,
    aspects: BTreeMap<(EntityId, AspectKey, FieldKey), WorthQueryObservedFieldRevision>,
    adjacencies: BTreeMap<(EntityId, KindId, u8), WorthQueryObservedAdjacencyRevision>,
    walks: Option<Arc<StructuralWalkDependencies>>,
    allocation: Option<Arc<SourceFactsAllocation>>,
}

pub(in crate::domain_computation::primary_graph) enum SourceFactReadDenial {
    Unavailable,
    WorkLimitExceeded,
    Execution(Cause),
}
impl SourceFactsBuilder {
    pub(in crate::domain_computation::primary_graph) fn observe_entity(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        entity: EntityId,
    ) -> Result<bool, Cause> {
        if !self.entities.contains(&entity)
            && let Some(allocation) = &self.allocation
        {
            allocation.claim_tree::<(EntityId, ())>(execution, self.entities.len())?;
        }
        Ok(self.entities.insert(entity))
    }
    pub(in crate::domain_computation::primary_graph) fn record_field(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        value: WorthQueryObservedFieldRevision,
    ) -> Result<bool, Cause> {
        if self.has_field(execution, value.entity, &value.aspect, &value.field)? {
            return Ok(false);
        }
        self.admit_field_record(execution, &value)?;
        self.aspects.insert(
            (value.entity, value.aspect.clone(), value.field.clone()),
            value,
        );
        Ok(true)
    }
    pub(in crate::domain_computation::primary_graph) fn record_adjacency(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        value: WorthQueryObservedAdjacencyRevision,
    ) -> Result<bool, Cause> {
        if !self.has_adjacency(value.anchor, value.relation_kind, value.direction) {
            self.admit_adjacency_record(execution, &value)?;
        }
        if let std::collections::btree_map::Entry::Vacant(entry) = self.adjacencies.entry((
            value.anchor,
            value.relation_kind,
            super::adjacency_direction_rank(value.direction),
        )) {
            entry.insert(value);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    pub(in crate::domain_computation::primary_graph) fn entities(
        &self,
    ) -> impl Iterator<Item = EntityId> + '_ {
        self.entities.iter().copied()
    }
    pub(in crate::domain_computation::primary_graph) fn source_count(&self) -> usize {
        self.entities.len()
            + self.aspects.len()
            + self.adjacencies.len()
            + usize::from(self.walks.is_some())
    }
    pub(in crate::domain_computation::primary_graph) fn walks(
        &self,
    ) -> Option<&Arc<StructuralWalkDependencies>> {
        self.walks.as_ref()
    }
    pub(in crate::domain_computation::primary_graph) fn set_walks(
        &mut self,
        walks: Arc<StructuralWalkDependencies>,
    ) {
        self.walks = Some(walks);
    }
    pub(in crate::domain_computation::primary_graph) fn record_walk(
        &mut self,
        walk: &Arc<StructuralWalkObservation>,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), Cause> {
        self.walks = Some(StructuralWalkDependencies::append(
            self.walks.as_ref(),
            walk,
            execution,
            context,
        )?);
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn has_field(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        entity: EntityId,
        aspect: &AspectKey,
        field: &FieldKey,
    ) -> Result<bool, Cause> {
        self.admit_field_lookup(execution, aspect, field)?;
        Ok(self
            .aspects
            .contains_key(&(entity, aspect.clone(), field.clone())))
    }
    pub(in crate::domain_computation::primary_graph) fn has_adjacency(
        &self,
        anchor: EntityId,
        kind: KindId,
        direction: RelationalAdjacencyDirection,
    ) -> bool {
        self.adjacencies
            .contains_key(&(anchor, kind, super::adjacency_direction_rank(direction)))
    }
    pub(in crate::domain_computation::primary_graph) fn observe_guard_field(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        projection: &VisibilityProjectionView,
        graph: &WorthQueryPrimaryGraphLayout,
        entity: EntityId,
        entity_name: &str,
        aspect: &AspectKey,
        field: &FieldKey,
    ) -> Result<WorthQueryObservedFieldRevision, SourceFactReadDenial> {
        let contract_revision = graph
            .aspect_contract(entity_name, aspect)
            .ok_or(SourceFactReadDenial::Unavailable)?
            .revision();
        self.admit_guard_field(execution, entity_name, aspect, field)
            .map_err(SourceFactReadDenial::Execution)?;
        match self.aspects.entry((entity, aspect.clone(), field.clone())) {
            std::collections::btree_map::Entry::Occupied(entry) => Ok(entry.get().clone()),
            std::collections::btree_map::Entry::Vacant(entry) => {
                let locator = AspectFieldLocator::new(
                    LocatorAuthority::Authoritative,
                    aspect.clone(),
                    CanonicalFieldPath::single(field.clone()),
                );
                let native_revision = projection
                    .entity_field_revision(execution, entity, &locator)
                    .map_err(SourceFactReadDenial::Execution)?;
                Ok(entry
                    .insert(WorthQueryObservedFieldRevision {
                        entity,
                        entity_name: entity_name.to_owned(),
                        aspect: aspect.clone(),
                        field: field.clone(),
                        contract_revision,
                        native_revision,
                    })
                    .clone())
            }
        }
    }
    pub(in crate::domain_computation::primary_graph) fn observe_adjacency(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        projection: &VisibilityProjectionView,
        anchor: EntityId,
        kind: KindId,
        direction: RelationalAdjacencyDirection,
    ) -> Result<WorthQueryObservedAdjacencyRevision, SourceFactReadDenial> {
        if !self.has_adjacency(anchor, kind, direction)
            && let Some(allocation) = &self.allocation
        {
            allocation
                .claim_tree::<((EntityId, KindId, u8), WorthQueryObservedAdjacencyRevision)>(
                    execution,
                    self.adjacencies.len(),
                )
                .map_err(SourceFactReadDenial::Execution)?;
        }
        match self
            .adjacencies
            .entry((anchor, kind, super::adjacency_direction_rank(direction)))
        {
            std::collections::btree_map::Entry::Occupied(entry) => Ok(entry.get().clone()),
            std::collections::btree_map::Entry::Vacant(entry) => {
                let revision = projection
                    .bounded_adjacency_structural_revision(execution, anchor, kind, direction, 1)
                    .map_err(|denial| match denial {
                        AdjacencyStructuralRevisionDenial::Execution(cause) => {
                            SourceFactReadDenial::Execution(cause)
                        }
                        AdjacencyStructuralRevisionDenial::WorkBudgetExceeded => {
                            SourceFactReadDenial::WorkLimitExceeded
                        }
                        AdjacencyStructuralRevisionDenial::AnchorUnavailable
                        | AdjacencyStructuralRevisionDenial::BasisUnavailable => {
                            SourceFactReadDenial::Unavailable
                        }
                    })?;
                Ok(entry
                    .insert(WorthQueryObservedAdjacencyRevision {
                        anchor,
                        relation_kind: kind,
                        direction,
                        native_revision: revision.revision(),
                        comparison_work_limit: revision.work_units(),
                        endpoints: Vec::new(),
                    })
                    .clone())
            }
        }
    }
    pub(in crate::domain_computation::primary_graph) fn finish(
        self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SourceFacts, Cause> {
        if let Some(allocation) = &self.allocation {
            allocation.claim_slice::<EntityId>(execution, self.entities.len())?;
            allocation
                .claim_slice::<WorthQueryObservedFieldRevision>(execution, self.aspects.len())?;
            allocation.claim_slice::<WorthQueryObservedAdjacencyRevision>(
                execution,
                self.adjacencies.len(),
            )?;
        }
        Ok(SourceFacts {
            entities: collect_facts(self.entities.into_iter())?,
            aspects: collect_facts(self.aspects.into_values())?,
            adjacencies: collect_facts(self.adjacencies.into_values())?,
            walks: self.walks,
            allocation: self.allocation,
        })
    }
}
fn collect_facts<T>(values: impl ExactSizeIterator<Item = T>) -> Result<Vec<T>, Cause> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(values.len())
        .map_err(|_| Cause::AllocationFailed)?;
    result.extend(values);
    Ok(result)
}
