//! Live populations carried by an immutable region and its selected root.
use crate::storage::overlay::PartitionState;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LiveRecordCounts {
    pub(crate) entities: usize,
    pub(crate) relations: usize,
}
impl LiveRecordCounts {
    pub(crate) fn from_partition(partition: &PartitionState) -> Self {
        Self {
            entities: partition.entity_arena.live_bitset.count_ones(),
            relations: partition.relation_arena.live_bitset.count_ones(),
        }
    }
    pub(crate) fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            entities: self.entities.checked_add(other.entities)?,
            relations: self.relations.checked_add(other.relations)?,
        })
    }
    pub(crate) fn checked_sub(self, other: Self) -> Option<Self> {
        Some(Self {
            entities: self.entities.checked_sub(other.entities)?,
            relations: self.relations.checked_sub(other.relations)?,
        })
    }
}
