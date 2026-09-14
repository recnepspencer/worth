use serde::{Deserialize, Serialize};

use crate::identity::data::{EntityId, KindId, RelationId};

use super::super::AspectFieldPatch;

/// Owner-sealed branch-local transition for reproducible record payload.
/// Public callers cannot construct these intents directly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterializationMutationIntent {
    SuspendEntity(SuspendEntityMaterializationIntent),
    SuspendRelation(SuspendRelationMaterializationIntent),
    RematerializeEntity(RematerializeEntityIntent),
    RematerializeRelation(RematerializeRelationIntent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuspendEntityMaterializationIntent {
    pub(crate) entity_id: EntityId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuspendRelationMaterializationIntent {
    pub(crate) relation_id: RelationId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RematerializeEntityIntent {
    pub(crate) entity_id: EntityId,
    pub(crate) kind_id: KindId,
    pub(crate) fields: AspectFieldPatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RematerializeRelationIntent {
    pub(crate) relation_id: RelationId,
    pub(crate) kind_id: KindId,
    pub(crate) source: EntityId,
    pub(crate) target: EntityId,
    pub(crate) fields: AspectFieldPatch,
}

impl MaterializationMutationIntent {
    pub(crate) fn record(&self) -> super::super::RecordRef {
        match self {
            Self::SuspendEntity(intent) => super::super::RecordRef::Entity(intent.entity_id),
            Self::SuspendRelation(intent) => super::super::RecordRef::Relation(intent.relation_id),
            Self::RematerializeEntity(intent) => super::super::RecordRef::Entity(intent.entity_id),
            Self::RematerializeRelation(intent) => {
                super::super::RecordRef::Relation(intent.relation_id)
            }
        }
    }

    pub(crate) fn partition_id(&self) -> crate::identity::data::PartitionId {
        match self.record() {
            super::super::RecordRef::Entity(entity) => entity.partition_id,
            super::super::RecordRef::Relation(relation) => relation.partition_id,
        }
    }

    pub(crate) const fn is_suspension(&self) -> bool {
        matches!(self, Self::SuspendEntity(_) | Self::SuspendRelation(_))
    }

    pub(crate) const fn is_rematerialization(&self) -> bool {
        matches!(
            self,
            Self::RematerializeEntity(_) | Self::RematerializeRelation(_)
        )
    }
}
