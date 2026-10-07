//! Owned allocation accounting for observed native source evidence.
use super::{
    WorthQueryObservedAdjacencyRevision, WorthQueryObservedFieldRevision,
    WorthQueryObservedSourceFootprint,
};
use worth_relational::facade::identity::EntityId;
impl WorthQueryObservedFieldRevision {
    pub(in crate::domain_computation::primary_graph::application_query) fn owned_capacity_bytes(
        &self,
    ) -> usize {
        let Self {
            entity: _entity,
            entity_name,
            aspect,
            field,
            contract_revision: _contract_revision,
            native_revision: _native_revision,
        } = self;
        entity_name
            .capacity()
            .saturating_add(aspect.owned_allocation_capacity_bytes())
            .saturating_add(field.owned_allocation_capacity_bytes())
    }
}
impl WorthQueryObservedAdjacencyRevision {
    pub(super) fn owned_capacity_bytes(&self) -> usize {
        let Self {
            anchor: _anchor,
            relation_kind: _relation_kind,
            direction: _direction,
            native_revision: _native_revision,
            comparison_work_limit: _comparison_work_limit,
            endpoints,
        } = self;
        endpoints
            .capacity()
            .saturating_mul(std::mem::size_of::<EntityId>())
    }
}
impl WorthQueryObservedSourceFootprint {
    pub(in crate::domain_computation::primary_graph) fn retained_bytes(&self) -> usize {
        // root_selection is separately shared/charged by the containing result.
        let Self {
            root: _root,
            complete: _complete,
            entities,
            aspects,
            adjacencies,
            root_selection: _root_selection,
        } = self;
        let entity_bytes = entities
            .capacity()
            .saturating_mul(std::mem::size_of::<EntityId>());
        let aspect_bytes = aspects.iter().fold(
            aspects
                .capacity()
                .saturating_mul(std::mem::size_of::<WorthQueryObservedFieldRevision>()),
            |bytes, aspect| bytes.saturating_add(aspect.owned_capacity_bytes()),
        );
        adjacencies.iter().fold(
            entity_bytes.saturating_add(aspect_bytes).saturating_add(
                adjacencies
                    .capacity()
                    .saturating_mul(std::mem::size_of::<WorthQueryObservedAdjacencyRevision>()),
            ),
            |bytes, adjacency| bytes.saturating_add(adjacency.owned_capacity_bytes()),
        )
    }
}
