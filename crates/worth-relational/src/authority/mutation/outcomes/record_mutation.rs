use crate::identity::data::{EntityId, KindId, RelationId};
use crate::storage::substrate::{EntityExtra, RelationExtra, StorageAllocation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RecordMutation {
    EntityCreated {
        entity_id: EntityId,
        kind_id: KindId,
        authoritative_patch: Option<worth_foundational::facade::AuthoritativeRecordAspectPatch>,
    },
    EntityUpdated {
        entity_id: EntityId,
        kind_id: KindId,
        old_authoritative_aspect_state: Option<StorageAllocation<EntityExtra>>,
        new_authoritative_aspect_state: Option<StorageAllocation<EntityExtra>>,
        authoritative_patch: Option<worth_foundational::facade::AuthoritativeRecordAspectPatch>,
    },
    EntityDeleted {
        entity_id: EntityId,
        kind_id: KindId,
        authoritative_patch: Option<worth_foundational::facade::AuthoritativeRecordAspectPatch>,
    },
    EntityMaterializationSuspended {
        entity_id: EntityId,
        kind_id: KindId,
        old_authoritative_aspect_state: Option<StorageAllocation<EntityExtra>>,
    },
    EntityRematerialized {
        entity_id: EntityId,
        kind_id: KindId,
        new_authoritative_aspect_state: Option<StorageAllocation<EntityExtra>>,
        authoritative_patch: Option<worth_foundational::facade::AuthoritativeRecordAspectPatch>,
    },
    RelationCreated {
        relation_id: RelationId,
        kind_id: KindId,
        source: EntityId,
        target: EntityId,
        authoritative_patch: Option<worth_foundational::facade::AuthoritativeRecordAspectPatch>,
    },
    RelationUpdated {
        relation_id: RelationId,
        kind_id: KindId,
        old_source: EntityId,
        old_target: EntityId,
        new_source: EntityId,
        new_target: EntityId,
        old_authoritative_aspect_state: Option<StorageAllocation<RelationExtra>>,
        new_authoritative_aspect_state: Option<StorageAllocation<RelationExtra>>,
        authoritative_patch: Option<worth_foundational::facade::AuthoritativeRecordAspectPatch>,
    },
    RelationDeleted {
        relation_id: RelationId,
        kind_id: KindId,
        source: EntityId,
        target: EntityId,
        authoritative_aspect_state: Option<StorageAllocation<RelationExtra>>,
    },
    RelationRetainedForAudit {
        relation_id: RelationId,
        kind_id: KindId,
        source: EntityId,
        target: EntityId,
        authoritative_aspect_state: Option<StorageAllocation<RelationExtra>>,
    },
    RelationMaterializationSuspended {
        relation_id: RelationId,
        kind_id: KindId,
        source: EntityId,
        target: EntityId,
        old_authoritative_aspect_state: Option<StorageAllocation<RelationExtra>>,
    },
    RelationRematerialized {
        relation_id: RelationId,
        kind_id: KindId,
        source: EntityId,
        target: EntityId,
        new_authoritative_aspect_state: Option<StorageAllocation<RelationExtra>>,
        authoritative_patch: Option<worth_foundational::facade::AuthoritativeRecordAspectPatch>,
    },
}
