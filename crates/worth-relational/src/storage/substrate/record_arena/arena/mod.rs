mod allocation;
mod allocation_delta;
mod allocation_walk;
mod merge;
mod payload_allocation;
mod pin_counts;
mod pinning;
mod slot_directory;
mod slot_lifecycle;
mod sparse_clone;

use pin_counts::RecordPinCounts;
pub(crate) use slot_directory::RecordSlotDirectory;

use crate::storage::substrate::SharedColumn;
use std::collections::BTreeMap;

use crate::identity::data::{KindId, PartitionId, RecordId, VersionId};
use crate::storage::data::RecordLifecycleState;
use crate::storage::partition::DenseSlotBitSet;
use crate::symbols::data::Symbol;

use super::{
    slot_view::SlotView, EntityRecordKind, LifecycleCounts, RecordKind, RelationRecordKind,
};

#[derive(Debug)]
pub(crate) struct SlotInit<K: RecordKind> {
    pub partition_id: PartitionId,
    pub kind_id: KindId,
    pub version_id: VersionId,
    pub extra: K::Extra,
}

#[derive(Debug, Clone)]
pub(crate) struct RecordArena<K: RecordKind> {
    pub(crate) slots: RecordSlotDirectory,
    pub(crate) partition_ids: SharedColumn<PartitionId>,
    pub(crate) generations: SharedColumn<u32>,
    pub(crate) lifecycle: SharedColumn<RecordLifecycleState>,
    pub(crate) kind_ids: SharedColumn<Option<KindId>>,
    pub(crate) metadata_history: SharedColumn<SharedColumn<K::Meta>>,
    pub(crate) created_at: SharedColumn<VersionId>,
    pub(crate) retired_at: SharedColumn<Option<VersionId>>,
    pub(crate) extra: SharedColumn<K::Extra>,
    pub(crate) aspect_versions: SharedColumn<BTreeMap<Symbol, u64>>,
    pub(crate) diagnostics_enrichment: SharedColumn<BTreeMap<Symbol, String>>,
    pub(crate) branch_pins: RecordPinCounts,
    pub(crate) replay_pins: RecordPinCounts,
    pub(crate) snapshot_pins: RecordPinCounts,
    pub(crate) live_bitset: DenseSlotBitSet,
    pub(crate) reclaimable_bitset: DenseSlotBitSet,
}

impl<K: RecordKind> RecordArena<K> {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: RecordSlotDirectory::with_capacity(capacity),
            partition_ids: SharedColumn::with_capacity(capacity),
            generations: SharedColumn::with_capacity(capacity),
            lifecycle: SharedColumn::with_capacity(capacity),
            kind_ids: SharedColumn::with_capacity(capacity),
            metadata_history: SharedColumn::with_capacity(capacity),
            created_at: SharedColumn::with_capacity(capacity),
            retired_at: SharedColumn::with_capacity(capacity),
            extra: SharedColumn::with_capacity(capacity),
            aspect_versions: SharedColumn::with_capacity(capacity),
            diagnostics_enrichment: SharedColumn::with_capacity(capacity),
            branch_pins: RecordPinCounts::new(),
            replay_pins: RecordPinCounts::new(),
            snapshot_pins: RecordPinCounts::new(),
            live_bitset: DenseSlotBitSet::with_capacity(capacity),
            reclaimable_bitset: DenseSlotBitSet::with_capacity(capacity),
        }
    }

    pub(crate) fn lifecycle_counts(&self) -> LifecycleCounts {
        lifecycle_counts(self.lifecycle.iter())
    }

    pub(crate) fn get(&self, id: &RecordId<K::Domain>) -> Option<SlotView<'_, K>> {
        let slot = super::slot_of::<K>(id);
        self.get_slot(slot).filter(|view| {
            view.generation() == super::generation_of::<K>(id)
                && view.partition_id() == super::partition_of::<K>(id)
        })
    }

    pub(crate) fn get_slot(&self, slot: usize) -> Option<SlotView<'_, K>> {
        self.slots
            .physical_index(slot)
            .map(|physical| SlotView::new(self, physical))
    }

    pub(crate) fn slot_count(&self) -> usize {
        self.slots.len()
    }

    pub(crate) fn occupied_slots(&self) -> Vec<usize> {
        self.slots.occupied_slots()
    }

    pub(crate) fn physical_index(&self, slot: usize) -> Option<usize> {
        self.slots.physical_index(slot)
    }

    pub(crate) fn retired_at_for_slot(&self, slot: usize) -> Option<VersionId> {
        self.physical_index(slot)
            .and_then(|physical| self.retired_at.get(physical).copied().flatten())
    }

    pub(crate) fn metadata_history_at(&self, slot: usize) -> Option<&SharedColumn<K::Meta>> {
        self.physical_index(slot)
            .and_then(|physical| self.metadata_history.get(physical))
    }

    pub(crate) fn metadata_history_at_mut(
        &mut self,
        slot: usize,
    ) -> Option<&mut SharedColumn<K::Meta>> {
        let physical = self.physical_index(slot)?;
        self.metadata_history.get_mut(physical)
    }

    pub(crate) fn aspect_versions_at(&self, slot: usize) -> Option<&BTreeMap<Symbol, u64>> {
        self.physical_index(slot)
            .and_then(|physical| self.aspect_versions.get(physical))
    }

    pub(crate) fn aspect_versions_at_mut(
        &mut self,
        slot: usize,
    ) -> Option<&mut BTreeMap<Symbol, u64>> {
        let physical = self.physical_index(slot)?;
        self.aspect_versions.get_mut(physical)
    }

    pub(crate) fn extra_at(&self, slot: usize) -> Option<&K::Extra> {
        self.physical_index(slot)
            .and_then(|physical| self.extra.get(physical))
    }

    pub(crate) fn created_at_for_slot(&self, slot: usize) -> Option<VersionId> {
        self.physical_index(slot)
            .and_then(|physical| self.created_at.get(physical).copied())
    }

    /// Canonical truth/lifecycle/index bytes only. Diagnostics, retention
    /// counters, and allocator bookkeeping have independent lifecycle lanes.
    #[cfg(test)]
    pub(crate) fn authoritative_allocation_bytes(&self) -> u64 {
        self.allocation_inventory().authoritative_bytes
    }
}

pub(crate) type EntityArena = RecordArena<EntityRecordKind>;
pub(crate) type RelationArena = RecordArena<RelationRecordKind>;

#[derive(Clone, Copy)]
pub(crate) enum PinClass {
    #[cfg(test)]
    Branch,
    Replay,
}

pub(crate) fn lifecycle_counts<'a>(
    lifecycle: impl Iterator<Item = &'a RecordLifecycleState>,
) -> LifecycleCounts {
    let mut counts = LifecycleCounts::default();
    for state in lifecycle {
        match state {
            RecordLifecycleState::Live => counts.live += 1,
            RecordLifecycleState::MaterializationUnavailable => counts.unavailable += 1,
            RecordLifecycleState::Reusable => counts.reusable += 1,
            RecordLifecycleState::DeletedRetained
            | RecordLifecycleState::RetainedDanglingForAudit
            | RecordLifecycleState::PinnedBySnapshot
            | RecordLifecycleState::PinnedByBranch
            | RecordLifecycleState::PinnedByReplayRetention
            | RecordLifecycleState::Reclaimable => counts.deleted += 1,
        }
    }
    counts
}
