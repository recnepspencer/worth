use super::{RelationalTransactionReadLocus as Read, RelationalTransactionWriteLocus as Write};
use crate::identity::data::{EntityId, RelationId};
use crate::transactions::data::{CreatedEntityRef, CreatedRelationRef};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum IndexKey {
    Entity(EntityId),
    Relation(RelationId),
    CreatedEntity(CreatedEntityRef),
    CreatedRelation(CreatedRelationRef),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct IntentLocation {
    pub(super) batch_index: usize,
    pub(super) intent_index: usize,
}
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct IndexRow {
    pub(super) key: IndexKey,
    pub(super) location: IntentLocation,
    /// Retains repeated Bulk occurrences at the same intent location.
    pub(super) ordinal: usize,
    pub(super) observes: bool,
}
impl IndexRow {
    pub(super) fn read(&self) -> Option<Read> {
        match (&self.key, self.observes) {
            (IndexKey::Entity(id), true) => Some(Read::Existing(
                crate::transactions::data::RecordRef::Entity(*id),
            )),
            _ => None,
        }
    }
    pub(super) fn write(&self) -> Option<Write> {
        if self.observes {
            return None;
        }
        Some(match &self.key {
            IndexKey::Entity(id) => {
                Write::Existing(crate::transactions::data::RecordRef::Entity(*id))
            }
            IndexKey::Relation(id) => {
                Write::Existing(crate::transactions::data::RecordRef::Relation(*id))
            }
            IndexKey::CreatedEntity(key) => Write::CreatedEntity(key.clone()),
            IndexKey::CreatedRelation(key) => Write::CreatedRelation(key.clone()),
        })
    }
    pub(super) fn partition(&self) -> Option<crate::identity::data::PartitionId> {
        if self.observes {
            return None;
        }
        Some(match &self.key {
            IndexKey::Entity(id) => id.partition_id,
            IndexKey::Relation(id) => id.partition_id,
            IndexKey::CreatedEntity(key) => key.partition_id,
            IndexKey::CreatedRelation(key) => key.partition_id,
        })
    }
}
