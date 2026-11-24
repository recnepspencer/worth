use crate::identity::data::EntityId;
use crate::storage::overlay::PartitionAccess;

use super::AdjacencyDirection;

/// Ordered borrowed candidates for a leased traversal. The caller checkpoints
/// each edge before materialization, so fanout never becomes a temporary Vec.
pub fn relation_candidates_from_state(
    state: &dyn PartitionAccess,
    entity_id: EntityId,
    direction: AdjacencyDirection,
) -> super::adjacency::AdjacencyIds<'_> {
    state
        .get_partition(entity_id.partition_id)
        .and_then(|partition| direction.table(partition).get(entity_id.slot_index()))
        .map(|relations| relations.current_ids())
        .unwrap_or_default()
}
