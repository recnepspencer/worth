use crate::contracts::identity::data::primitives::{KindId, PartitionId};
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MutationEvent {
    EntityCreated {
        entity_id: EntityId,
        kind_id: KindId,
    },
    BulkEntitiesCreated {
        partition_id: PartitionId,
        kind_id: KindId,
        count: usize,
    },
    EntityUpdated {
        entity_id: EntityId,
    },
    EntityReplaced {
        replaced_entity_id: EntityId,
        replacement_entity_id: EntityId,
        kind_id: KindId,
    },
    EntityDeleted {
        entity_id: EntityId,
    },
    RelationCreated {
        relation_id: RelationId,
        source: EntityId,
        target: EntityId,
        kind_id: KindId,
    },
    RelationUpdated {
        relation_id: RelationId,
    },
    BulkRelationsCreated {
        partition_id: PartitionId,
        kind_id: KindId,
        count: usize,
    },
    RelationDeleted {
        relation_id: RelationId,
    },
}
