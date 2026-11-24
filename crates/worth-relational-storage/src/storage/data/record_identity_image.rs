//! Immutable source allocator image; clones retain the same data and admission.
use crate::identity::data::PartitionId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;

#[derive(Debug, Clone)]
pub struct DurableRecordIdentityState(Arc<RecordIdentityImageData>);

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename = "DurableRecordIdentityState", deny_unknown_fields)]
pub struct RecordIdentityImageData {
    pub schema_version: u16,
    #[serde(default)]
    pub generation_high_water: Vec<DurableRecordGenerationHighWater>,
    #[serde(default)]
    pub reusable_slots: Vec<DurableReusableRecordSlot>,
    #[serde(default)]
    pub append_frontiers: Vec<DurableRecordSlotFrontier>,
    #[serde(default)]
    pub pending_reservations: Vec<DurablePendingRecordReservation>,
    #[serde(skip)]
    _memory: Option<ExecutionMemoryReservation>,
}

impl DurableRecordIdentityState {
    pub const CURRENT_SCHEMA_VERSION: u16 = 2;

    pub fn current_with_memory(
        generation_high_water: Vec<DurableRecordGenerationHighWater>,
        reusable_slots: Vec<DurableReusableRecordSlot>,
        append_frontiers: Vec<DurableRecordSlotFrontier>,
        pending_reservations: Vec<DurablePendingRecordReservation>,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Self {
        Self(Arc::new(RecordIdentityImageData {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            generation_high_water,
            reusable_slots,
            append_frontiers,
            pending_reservations,
            _memory: memory,
        }))
    }

    /// The actual private Arc allocation, excluding the four array backings.
    pub fn control_storage_bytes() -> Option<usize> {
        let layout = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::new::<RecordIdentityImageData>())
            .ok()?
            .0
            .pad_to_align();
        Some(layout.size())
    }
}

impl std::ops::Deref for DurableRecordIdentityState {
    type Target = RecordIdentityImageData;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Default for DurableRecordIdentityState {
    fn default() -> Self {
        Self(Arc::new(RecordIdentityImageData::default()))
    }
}
impl PartialEq for DurableRecordIdentityState {
    fn eq(&self, other: &Self) -> bool {
        self.schema_version == other.schema_version
            && self.generation_high_water == other.generation_high_water
            && self.reusable_slots == other.reusable_slots
            && self.append_frontiers == other.append_frontiers
            && self.pending_reservations == other.pending_reservations
    }
}
impl Eq for DurableRecordIdentityState {}
impl Serialize for DurableRecordIdentityState {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for DurableRecordIdentityState {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        RecordIdentityImageData::deserialize(deserializer).map(|data| Self(Arc::new(data)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DurableRecordGenerationClass {
    Entity,
    Relation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DurableRecordGenerationHighWater {
    pub class: DurableRecordGenerationClass,
    pub partition_id: PartitionId,
    pub slot: u64,
    pub generation: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableReusableRecordSlot {
    pub class: DurableRecordGenerationClass,
    pub partition_id: PartitionId,
    pub slot: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DurableRecordSlotFrontier {
    pub class: DurableRecordGenerationClass,
    pub partition_id: PartitionId,
    pub next_slot: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DurableRecordReservationOrigin {
    AppendFrontier,
    Reclaimed { prior_generation: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurablePendingRecordReservation {
    pub class: DurableRecordGenerationClass,
    pub partition_id: PartitionId,
    pub slot: u64,
    pub generation: u32,
    pub origin: DurableRecordReservationOrigin,
}
