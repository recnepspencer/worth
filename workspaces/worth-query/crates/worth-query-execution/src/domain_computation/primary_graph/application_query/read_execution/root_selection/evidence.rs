use crate::domain_computation::primary_graph::structural_walk::{
    StructuralWalkDependencies, StructuralWalkObservation,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_foundational::facade::{AspectKey, FieldKey};
use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::runtime::{
    AdjacencyStructuralRevisionDenial, RelationalAdjacencyDirection, VisibilityProjectionView,
};

use super::{read_execution_denial, RootSelectionWork, WorthQueryApplicationReadExecutionDenial};
use crate::domain_computation::primary_graph::application_query::{
    observed_source::{
        WorthQueryObservedAdjacencyRevision, WorthQueryObservedFieldRevision,
        WorthQueryObservedRootSelection,
    },
    read_execution::WorthQueryApplicationReadExecutionDenialKind,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;

mod allocation;
#[derive(Default)]
pub(super) struct RootPathSourceBuilder {
    entities: BTreeSet<EntityId>,
    aspects: BTreeMap<(EntityId, AspectKey, FieldKey), WorthQueryObservedFieldRevision>,
    adjacencies: BTreeMap<(EntityId, KindId, u8), WorthQueryObservedAdjacencyRevision>,
    walks: Option<Arc<StructuralWalkDependencies>>,
    allocation: Option<Arc<crate::domain_computation::primary_graph::application_query::observed_source::RootPathAllocation>>,
}

impl RootPathSourceBuilder {
    pub(super) fn observe_entity(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
    ) -> Result<bool, WorthQueryApplicationReadExecutionDenial> {
        if !self.entities.contains(&entity)
            && let Some(allocation) = &self.allocation
        {
            allocation.claim_tree::<(EntityId, ())>(execution, self.entities.len())?;
        }
        Ok(self.entities.insert(entity))
    }

    pub(super) fn record_field(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        aspect: WorthQueryObservedFieldRevision,
    ) -> Result<bool, WorthQueryApplicationReadExecutionDenial> {
        if let Some(allocation) = &self.allocation {
            let bytes = aspect
                .aspect
                .owned_allocation_capacity_bytes()
                .checked_add(aspect.field.owned_allocation_capacity_bytes())
                .and_then(|n| u64::try_from(n).ok())
                .ok_or_else(|| source_denial("source field key capacity"))?;
            allocation.claim(execution, bytes)?;
            allocation.claim_tree::<(
                (EntityId, AspectKey, FieldKey),
                WorthQueryObservedFieldRevision,
            )>(execution, self.aspects.len())?;
        }
        if let std::collections::btree_map::Entry::Vacant(entry) =
            self.aspects
                .entry((aspect.entity, aspect.aspect.clone(), aspect.field.clone()))
        {
            entry.insert(aspect);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub(super) fn record_adjacency(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        adjacency: WorthQueryObservedAdjacencyRevision,
    ) -> Result<bool, WorthQueryApplicationReadExecutionDenial> {
        let direction = match adjacency.direction {
            RelationalAdjacencyDirection::Outgoing => 0,
            RelationalAdjacencyDirection::Incoming => 1,
        };
        if !self
            .adjacencies
            .contains_key(&(adjacency.anchor, adjacency.relation_kind, direction))
            && let Some(allocation) = &self.allocation
        {
            allocation
                .claim_tree::<((EntityId, KindId, u8), WorthQueryObservedAdjacencyRevision)>(
                    execution,
                    self.adjacencies.len(),
                )?;
        }
        if let std::collections::btree_map::Entry::Vacant(entry) =
            self.adjacencies
                .entry((adjacency.anchor, adjacency.relation_kind, direction))
        {
            entry.insert(adjacency);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub(super) fn entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.entities.iter().copied()
    }

    pub(super) fn copy_cost(&self) -> usize {
        self.entities.len()
            + self.aspects.len()
            + self.adjacencies.len()
            + usize::from(self.walks.is_some())
    }

    pub(super) fn walks(&self) -> Option<&Arc<StructuralWalkDependencies>> {
        self.walks.as_ref()
    }

    pub(super) fn set_walks(&mut self, walks: Arc<StructuralWalkDependencies>) {
        self.walks = Some(walks);
    }

    pub(super) fn record_walk(
        &mut self,
        walk: &Arc<StructuralWalkObservation>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        self.walks = Some(StructuralWalkDependencies::append(
            self.walks.as_ref(),
            walk,
            execution,
            context,
        )?);
        Ok(())
    }

    pub(super) fn observe_guard_field(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        projection: &VisibilityProjectionView,
        graph: &WorthQueryPrimaryGraphLayout,
        entity: EntityId,
        entity_name: &str,
        aspect: &AspectKey,
        field: &FieldKey,
        work: &mut RootSelectionWork,
    ) -> Result<WorthQueryObservedFieldRevision, WorthQueryApplicationReadExecutionDenial> {
        let contract_revision = graph
            .aspect_contract(entity_name, aspect)
            .ok_or_else(|| source_denial(entity_name))?
            .revision();
        if let Some(allocation) = &self.allocation {
            let names = aspect
                .owned_allocation_capacity_bytes()
                .checked_add(field.owned_allocation_capacity_bytes())
                .and_then(|n| u64::try_from(n).ok())
                .ok_or_else(|| source_denial(entity_name))?;
            // Index keys, stored/returned names and temporary field locator.
            let bytes = names
                .checked_mul(6)
                .and_then(|n| n.checked_add(2 * entity_name.len() as u64))
                .and_then(|n| {
                    n.checked_add(
                        (size_of::<worth_foundational::facade::AspectFieldLocator>()
                            + 8 * size_of::<usize>()) as u64,
                    )
                })
                .ok_or_else(|| source_denial(entity_name))?;
            allocation.claim(execution, bytes)?;
            allocation.claim_tree::<(
                (EntityId, AspectKey, FieldKey),
                WorthQueryObservedFieldRevision,
            )>(execution, self.aspects.len())?;
        }
        match self.aspects.entry((entity, aspect.clone(), field.clone())) {
            std::collections::btree_map::Entry::Occupied(entry) => Ok(entry.get().clone()),
            std::collections::btree_map::Entry::Vacant(entry) => {
                work.charge_source_observation(entity_name)?;
                let locator = worth_foundational::facade::AspectFieldLocator::new(
                    worth_foundational::facade::LocatorAuthority::Authoritative,
                    aspect.clone(),
                    worth_foundational::facade::CanonicalFieldPath::single(field.clone()),
                );
                let native_revision = projection.entity_field_revision(execution, entity, &locator).map_err(|cause| crate::domain_computation::primary_graph::application_query::read_execution::denial::relational_read_denial(cause, entity_name))?;
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

    pub(super) fn observe_adjacencies(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        projection: &VisibilityProjectionView,
        anchor: EntityId,
        relation_kind: KindId,
        direction: RelationalAdjacencyDirection,
        subject: &str,
        work: &mut RootSelectionWork,
    ) -> Result<WorthQueryObservedAdjacencyRevision, WorthQueryApplicationReadExecutionDenial> {
        let direction_key = match direction {
            RelationalAdjacencyDirection::Outgoing => 0,
            RelationalAdjacencyDirection::Incoming => 1,
        };
        if !self
            .adjacencies
            .contains_key(&(anchor, relation_kind, direction_key))
            && let Some(allocation) = &self.allocation
        {
            allocation
                .claim_tree::<((EntityId, KindId, u8), WorthQueryObservedAdjacencyRevision)>(
                    execution,
                    self.adjacencies.len(),
                )?;
        }
        match self
            .adjacencies
            .entry((anchor, relation_kind, direction_key))
        {
            std::collections::btree_map::Entry::Occupied(entry) => Ok(entry.get().clone()),
            std::collections::btree_map::Entry::Vacant(entry) => {
                work.charge_source_observation(subject)?;
                let revision = projection
                    .bounded_adjacency_structural_revision(
                        execution,
                        anchor,
                        relation_kind,
                        direction,
                        1,
                    )
                    .map_err(|unavailable| match unavailable {
                        AdjacencyStructuralRevisionDenial::Execution(cause) => crate::domain_computation::primary_graph::application_query::read_execution::denial::relational_read_denial(cause, subject),
                        AdjacencyStructuralRevisionDenial::WorkBudgetExceeded => {
                            read_execution_denial(
                                WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
                                subject,
                            )
                        }
                        AdjacencyStructuralRevisionDenial::AnchorUnavailable
                        | AdjacencyStructuralRevisionDenial::BasisUnavailable => {
                            source_denial(subject)
                        }
                    })?;
                Ok(entry
                    .insert(WorthQueryObservedAdjacencyRevision {
                        anchor,
                        relation_kind,
                        direction,
                        native_revision: revision.revision(),
                        comparison_work_limit: revision.work_units(),
                        endpoints: Vec::new(),
                    })
                    .clone())
            }
        }
    }

    pub(super) fn finish(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<WorthQueryObservedRootSelection>, WorthQueryApplicationReadExecutionDenial>
    {
        if let Some(allocation) = &self.allocation {
            let bytes = self
                .entities
                .len()
                .checked_mul(size_of::<EntityId>())
                .and_then(|n| {
                    n.checked_add(
                        self.aspects
                            .len()
                            .checked_mul(size_of::<WorthQueryObservedFieldRevision>())?,
                    )
                })
                .and_then(|n| {
                    n.checked_add(
                        self.adjacencies
                            .len()
                            .checked_mul(size_of::<WorthQueryObservedAdjacencyRevision>())?,
                    )
                })
                .and_then(|n| {
                    n.checked_add(
                        size_of::<WorthQueryObservedRootSelection>() + 2 * size_of::<usize>(),
                    )
                })
                .and_then(|n| u64::try_from(n).ok())
                .ok_or_else(|| source_denial("source witness capacity"))?;
            allocation.claim(execution, bytes)?;
        }
        let mut entities = Vec::with_capacity(self.entities.len());
        entities.extend(self.entities);
        let mut aspects = Vec::with_capacity(self.aspects.len());
        aspects.extend(self.aspects.into_values());
        let mut adjacencies = Vec::with_capacity(self.adjacencies.len());
        adjacencies.extend(self.adjacencies.into_values());
        let source = WorthQueryObservedRootSelection::new(
            entities,
            aspects,
            adjacencies,
            self.walks,
            self.allocation,
        );
        Ok(Arc::new(source))
    }
}

fn source_denial(subject: &str) -> WorthQueryApplicationReadExecutionDenial {
    read_execution_denial(
        WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
        subject,
    )
}
