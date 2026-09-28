use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_foundational::facade::{AspectKey, FieldKey};
use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::runtime::{RelationalAdjacencyDirection, VisibilityProjectionView};

use super::{read_execution_denial, RootSelectionWork, WorthQueryApplicationReadExecutionDenial};
use crate::domain_computation::primary_graph::application_query::{
    observed_source::{
        WorthQueryObservedAdjacencyRevision, WorthQueryObservedFieldRevision,
        WorthQueryObservedRootSelection,
    },
    read_execution::WorthQueryApplicationReadExecutionDenialKind,
    resource_lifecycle::WorthQueryApplicationResultBufferReservation,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;

#[derive(Clone, Default)]
pub(super) struct RootPathSourceBuilder {
    entities: BTreeSet<EntityId>,
    aspects: BTreeMap<(EntityId, AspectKey, FieldKey), WorthQueryObservedFieldRevision>,
    adjacencies: BTreeMap<(EntityId, KindId, u8), WorthQueryObservedAdjacencyRevision>,
}

impl RootPathSourceBuilder {
    pub(super) fn observe_entity(&mut self, entity: EntityId) -> bool {
        self.entities.insert(entity)
    }

    pub(super) fn record_field(&mut self, aspect: WorthQueryObservedFieldRevision) -> bool {
        if let std::collections::btree_map::Entry::Vacant(entry) =
            self.aspects
                .entry((aspect.entity, aspect.aspect.clone(), aspect.field.clone()))
        {
            entry.insert(aspect);
            true
        } else {
            false
        }
    }

    pub(super) fn record_adjacency(
        &mut self,
        adjacency: WorthQueryObservedAdjacencyRevision,
    ) -> bool {
        let direction = match adjacency.direction {
            RelationalAdjacencyDirection::Outgoing => 0,
            RelationalAdjacencyDirection::Incoming => 1,
        };
        if let std::collections::btree_map::Entry::Vacant(entry) =
            self.adjacencies
                .entry((adjacency.anchor, adjacency.relation_kind, direction))
        {
            entry.insert(adjacency);
            true
        } else {
            false
        }
    }

    pub(super) fn entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.entities.iter().copied()
    }

    pub(super) fn copy_cost(&self) -> usize {
        self.entities.len() + self.aspects.len() + self.adjacencies.len()
    }

    pub(super) fn observe_guard_field(
        &mut self,
        projection: &VisibilityProjectionView<'_>,
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
        match self.aspects.entry((entity, aspect.clone(), field.clone())) {
            std::collections::btree_map::Entry::Occupied(entry) => Ok(entry.get().clone()),
            std::collections::btree_map::Entry::Vacant(entry) => {
                work.charge_source_observation(entity_name)?;
                let locator = worth_foundational::facade::AspectFieldLocator::new(
                    worth_foundational::facade::LocatorAuthority::Authoritative,
                    aspect.clone(),
                    worth_foundational::facade::CanonicalFieldPath::single(field.clone()),
                );
                let native_revision = projection.entity_field_revision(entity, &locator);
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
        projection: &VisibilityProjectionView<'_>,
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
        match self
            .adjacencies
            .entry((anchor, relation_kind, direction_key))
        {
            std::collections::btree_map::Entry::Occupied(entry) => Ok(entry.get().clone()),
            std::collections::btree_map::Entry::Vacant(entry) => {
                work.charge_source_observation(subject)?;
                let revision = projection
                    .bounded_adjacency_structural_revision(anchor, relation_kind, direction, 1)
                    .map_err(|_| source_denial(subject))?;
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
        result_buffer: &mut WorthQueryApplicationResultBufferReservation,
    ) -> Result<Arc<WorthQueryObservedRootSelection>, WorthQueryApplicationReadExecutionDenial>
    {
        let retained_bytes = std::mem::size_of::<WorthQueryObservedRootSelection>()
            .saturating_add(2 * std::mem::size_of::<usize>())
            .saturating_add(
                self.entities
                    .len()
                    .saturating_mul(std::mem::size_of::<EntityId>()),
            )
            .saturating_add(
                self.aspects
                    .len()
                    .saturating_mul(std::mem::size_of::<WorthQueryObservedFieldRevision>()),
            )
            .saturating_add(self.aspects.values().fold(0usize, |bytes, aspect| {
                bytes
                    .saturating_add(aspect.entity_name.capacity())
                    .saturating_add(aspect.aspect.as_str().len())
                    .saturating_add(aspect.field.as_str().len())
            }))
            .saturating_add(
                self.adjacencies
                    .len()
                    .saturating_mul(std::mem::size_of::<WorthQueryObservedAdjacencyRevision>()),
            );
        result_buffer
            .claim(retained_bytes)
            .map_err(|()| buffer_denial())?;
        let mut entities = Vec::with_capacity(self.entities.len());
        entities.extend(self.entities);
        let mut aspects = Vec::with_capacity(self.aspects.len());
        aspects.extend(self.aspects.into_values());
        let mut adjacencies = Vec::with_capacity(self.adjacencies.len());
        adjacencies.extend(self.adjacencies.into_values());
        let source = WorthQueryObservedRootSelection::new(entities, aspects, adjacencies);
        if source.retained_bytes() != retained_bytes {
            return Err(buffer_denial());
        }
        Ok(Arc::new(source))
    }
}

fn buffer_denial() -> WorthQueryApplicationReadExecutionDenial {
    read_execution_denial(
        WorthQueryApplicationReadExecutionDenialKind::ResultBufferLimitExceeded,
        "root-path-source",
    )
}

fn source_denial(subject: &str) -> WorthQueryApplicationReadExecutionDenial {
    read_execution_denial(
        WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
        subject,
    )
}
