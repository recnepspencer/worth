//! Immutable source allocator image; clones retain the same data and admission.
use super::{
    DurablePendingRecordReservation, DurableRecordGenerationHighWater, DurableRecordSlotFrontier,
    DurableReusableRecordSlot,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;

#[derive(Debug, Clone)]
pub(crate) struct DurableRecordIdentityState(Arc<RecordIdentityImageData>);

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename = "DurableRecordIdentityState", deny_unknown_fields)]
pub(crate) struct RecordIdentityImageData {
    pub(crate) schema_version: u16,
    #[serde(default)]
    pub(crate) generation_high_water: Vec<DurableRecordGenerationHighWater>,
    #[serde(default)]
    pub(crate) reusable_slots: Vec<DurableReusableRecordSlot>,
    #[serde(default)]
    pub(crate) append_frontiers: Vec<DurableRecordSlotFrontier>,
    #[serde(default)]
    pub(crate) pending_reservations: Vec<DurablePendingRecordReservation>,
    #[serde(skip)]
    _memory: Option<ExecutionMemoryReservation>,
}

impl DurableRecordIdentityState {
    pub(crate) const CURRENT_SCHEMA_VERSION: u16 = 2;

    pub(crate) fn current_with_memory(
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
    pub(crate) fn control_storage_bytes() -> Option<usize> {
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
