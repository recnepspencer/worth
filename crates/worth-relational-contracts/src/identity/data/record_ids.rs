use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use super::{Generation, LocalSlot, PartitionId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntityDomain {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelationDomain {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StorageId<K> {
    pub partition_id: PartitionId,
    pub local_slot: LocalSlot,
    #[serde(skip)]
    _marker: PhantomData<K>,
}

impl<K> StorageId<K> {
    pub const fn new(partition_id: PartitionId, local_slot: u64) -> Self {
        Self {
            partition_id,
            local_slot: LocalSlot::new(local_slot),
            _marker: PhantomData,
        }
    }

    pub const fn partition_value(self) -> u32 {
        self.partition_id.as_u32()
    }

    pub const fn partition_value_u64(self) -> u64 {
        self.partition_id.as_u64()
    }

    pub const fn local_slot_value(self) -> u64 {
        self.local_slot.as_u64()
    }

    pub const fn slot_index(self) -> usize {
        self.local_slot.index()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RecordId<K> {
    pub partition_id: PartitionId,
    pub local_slot: LocalSlot,
    pub generation: Generation,
    #[serde(skip)]
    _marker: PhantomData<K>,
}

impl<K> RecordId<K> {
    pub const fn new(partition_id: PartitionId, local_slot: u64, generation: u32) -> Self {
        Self {
            partition_id,
            local_slot: LocalSlot::new(local_slot),
            generation: Generation::new(generation),
            _marker: PhantomData,
        }
    }

    pub const fn storage_id(&self) -> StorageId<K> {
        StorageId {
            partition_id: self.partition_id,
            local_slot: self.local_slot,
            _marker: PhantomData,
        }
    }

    pub const fn partition_value(self) -> u32 {
        self.partition_id.as_u32()
    }

    pub const fn partition_value_u64(self) -> u64 {
        self.partition_id.as_u64()
    }

    pub const fn local_slot_value(self) -> u64 {
        self.local_slot.as_u64()
    }

    pub const fn slot_index(self) -> usize {
        self.local_slot.index()
    }

    pub const fn generation_value(self) -> u32 {
        self.generation.as_u32()
    }
}

pub type EntityId = RecordId<EntityDomain>;
pub type RelationId = RecordId<RelationDomain>;
pub type EntityStorageId = StorageId<EntityDomain>;
pub type RelationStorageId = StorageId<RelationDomain>;

#[cfg(test)]
mod tests {
    use super::{EntityId, EntityStorageId};
    use crate::identity::data::PartitionId;

    #[test]
    fn record_identity_accessors_expose_named_slot_and_generation_surfaces() {
        let entity_id = EntityId::new(PartitionId::new(3), 11, 7);
        let storage_id = EntityStorageId::new(PartitionId::new(3), 11);

        assert_eq!(entity_id.partition_value(), 3);
        assert_eq!(entity_id.partition_value_u64(), 3);
        assert_eq!(entity_id.local_slot_value(), 11);
        assert_eq!(entity_id.slot_index(), 11usize);
        assert_eq!(entity_id.generation_value(), 7);
        assert_eq!(storage_id.partition_value(), 3);
        assert_eq!(storage_id.partition_value_u64(), 3);
        assert_eq!(storage_id.local_slot_value(), 11);
        assert_eq!(storage_id.slot_index(), 11usize);
        assert_eq!(entity_id.storage_id(), storage_id);
    }
}
