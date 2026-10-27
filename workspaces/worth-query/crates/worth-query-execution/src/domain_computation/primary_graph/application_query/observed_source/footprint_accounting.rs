//! Owned allocation accounting for observed native source evidence.
use super::{
    WorthQueryObservedAdjacencyRevision, WorthQueryObservedFieldRevision,
    WorthQueryObservedSourceFootprint,
};
use worth_relational::facade::identity::EntityId;
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
