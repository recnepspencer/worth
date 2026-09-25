use worth_relational::facade::identity::EntityId;

use super::{
    WorthQueryObservedAdjacencyRevision, WorthQueryObservedFieldRevision,
    WorthQueryObservedSourceFootprint,
};

impl WorthQueryObservedSourceFootprint {
    pub(in crate::domain_computation::primary_graph) fn retained_bytes(&self) -> usize {
        let entity_bytes = self
            .entities
            .capacity()
            .saturating_mul(std::mem::size_of::<EntityId>());
        let aspect_bytes = self
            .aspects
            .capacity()
            .saturating_mul(std::mem::size_of::<WorthQueryObservedFieldRevision>());
        let aspect_text = self.aspects.iter().fold(0usize, |bytes, aspect| {
            bytes
                .saturating_add(aspect.entity_name.capacity())
                .saturating_add(aspect.aspect.as_str().len())
                .saturating_add(aspect.field.as_str().len())
        });
        let adjacency_bytes = self
            .adjacencies
            .capacity()
            .saturating_mul(std::mem::size_of::<WorthQueryObservedAdjacencyRevision>());
        let endpoint_bytes = self.adjacencies.iter().fold(0usize, |bytes, adjacency| {
            bytes.saturating_add(
                adjacency
                    .endpoints
                    .capacity()
                    .saturating_mul(std::mem::size_of::<EntityId>()),
            )
        });
        entity_bytes
            .saturating_add(aspect_bytes)
            .saturating_add(aspect_text)
            .saturating_add(adjacency_bytes)
            .saturating_add(endpoint_bytes)
    }
}
