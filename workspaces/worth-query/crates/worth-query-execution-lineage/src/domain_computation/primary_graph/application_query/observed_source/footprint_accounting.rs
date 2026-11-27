//! Owned allocation accounting for observed native source evidence.
use super::{
    WorthQueryObservedAdjacencyRevision, WorthQueryObservedFieldRevision,
    WorthQueryObservedSourceFootprint,
};
use worth_relational::facade::identity::EntityId;
impl WorthQueryObservedSourceFootprint {
    pub fn retained_bytes(&self) -> usize {
        // root_selection is separately shared/charged by the containing result.
        let Self {
            root: _root,
            complete: _complete,
            entities,
            aspects,
            adjacencies,
            root_selection: _root_selection,
            member_groups,
        } = self;
        let entity_bytes = entities
            .capacity()
            .saturating_mul(std::mem::size_of::<EntityId>());
        let member_bytes = member_groups.iter().fold(
            member_groups.capacity().saturating_mul(std::mem::size_of::<
                super::member_groups::WorthQueryObservedMemberGroups,
            >()),
            |bytes, member| {
                bytes.saturating_add(usize::try_from(member.owned_bytes()).unwrap_or(usize::MAX))
            },
        );
        let entity_bytes = entity_bytes.saturating_add(member_bytes);
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
