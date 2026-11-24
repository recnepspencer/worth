pub mod allocation;
#[cfg(test)]
pub use allocation::RecordArenaAllocationInventory;
#[cfg(test)]
pub use extra_allocations::extra_delta_resolve_scans_on_current_thread;
pub use extra_allocations::{ExtraDeltaCollector, SealedAllocationWatermarks};
pub mod entity_lineage;
pub mod extra_allocations;
pub use entity_lineage::ExtraLineageDenial;
pub mod allocation_delta;
pub mod allocation_walk;
pub mod historical_selection;
pub mod materialization_quote;
pub mod merge;
pub mod payload_allocation;
pub mod pin_counts;
pub mod pinning;
pub mod revision_maps;
pub mod slot_adoption;
pub use revision_maps::{AspectVersions, FieldRevisions};
pub mod slot_directory;
pub mod slot_lifecycle;
pub use slot_lifecycle::MetadataWriteDenial;
pub mod slot_visitation;
pub mod sparse_clone;

use pin_counts::RecordPinCounts;
pub use slot_directory::RecordSlotDirectory;
pub use slot_visitation::RecordSlotVisit;

use crate::storage::substrate::{SharedColumn, StorageAllocation};
use std::collections::BTreeMap;

use crate::identity::data::{KindId, PartitionId, RecordId, VersionId};
use crate::storage::data::RecordLifecycleState;
use crate::storage::partition::DenseSlotBitSet;
use crate::symbols::data::Symbol;

use super::{
    slot_view::SlotView, EntityRecordKind, LifecycleCounts, RecordKind, RelationRecordKind,
};

#[derive(Debug)]
pub struct SlotInit<K: RecordKind> {
    pub partition_id: PartitionId,
    pub kind_id: KindId,
    pub version_id: VersionId,
    pub extra: K::Extra,
}

#[derive(Debug, Clone)]
pub struct RecordArena<K: RecordKind> {
    pub slots: RecordSlotDirectory,
    pub partition_ids: SharedColumn<PartitionId>,
    pub generations: SharedColumn<u32>,
    pub lifecycle: SharedColumn<RecordLifecycleState>,
    pub kind_ids: SharedColumn<Option<KindId>>,
    pub metadata_history: SharedColumn<SharedColumn<StorageAllocation<K::Meta>>>,
    pub history_prefixes: SharedColumn<StorageAllocation<Option<super::RecordHistoryPrefix>>>,
    pub created_at: SharedColumn<VersionId>,
    pub retired_at: SharedColumn<Option<VersionId>>,
    pub extra: SharedColumn<StorageAllocation<K::Extra>>,
    pub aspect_versions: SharedColumn<AspectVersions>,
    pub field_revisions: SharedColumn<Option<FieldRevisions>>,
    pub diagnostics_enrichment: SharedColumn<StorageAllocation<BTreeMap<Symbol, String>>>,
    pub branch_pins: RecordPinCounts,
    pub replay_pins: RecordPinCounts,
    pub snapshot_pins: RecordPinCounts,
    pub live_bitset: DenseSlotBitSet,
    pub reclaimable_bitset: DenseSlotBitSet,
}

impl<K: RecordKind> RecordArena<K> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: RecordSlotDirectory::with_capacity(capacity),
            partition_ids: SharedColumn::with_capacity(capacity),
            generations: SharedColumn::with_capacity(capacity),
            lifecycle: SharedColumn::with_capacity(capacity),
            kind_ids: SharedColumn::with_capacity(capacity),
            metadata_history: SharedColumn::with_capacity(capacity),
            history_prefixes: SharedColumn::with_capacity(capacity),
            created_at: SharedColumn::with_capacity(capacity),
            retired_at: SharedColumn::with_capacity(capacity),
            extra: SharedColumn::with_capacity(capacity),
            aspect_versions: SharedColumn::with_capacity(capacity),
            field_revisions: SharedColumn::with_capacity(capacity),
            diagnostics_enrichment: SharedColumn::with_capacity(capacity),
            branch_pins: RecordPinCounts::new(),
            replay_pins: RecordPinCounts::new(),
            snapshot_pins: RecordPinCounts::new(),
            live_bitset: DenseSlotBitSet::with_capacity(capacity),
            reclaimable_bitset: DenseSlotBitSet::with_capacity(capacity),
        }
    }

    pub fn lifecycle_counts(&self) -> LifecycleCounts {
        lifecycle_counts(self.lifecycle.iter())
    }

    pub fn get(&self, id: &RecordId<K::Domain>) -> Option<SlotView<'_, K>> {
        let slot = super::slot_of::<K>(id);
        self.get_slot(slot).filter(|view| {
            view.generation() == super::generation_of::<K>(id)
                && view.partition_id() == super::partition_of::<K>(id)
        })
    }

    pub fn get_slot(&self, slot: usize) -> Option<SlotView<'_, K>> {
        self.slots
            .physical_index(slot)
            .map(|physical| SlotView::new(self, physical))
    }

    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    pub fn try_occupied_slots<Stop>(
        &self,
        prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<
        std::iter::Map<
            crate::storage::substrate::SharedMapIter<'_, u64, usize>,
            fn((&u64, &usize)) -> usize,
        >,
        Stop,
    > {
        Ok(self.slots.try_occupied_slots(prepare)?)
    }

    pub fn occupied_slots(&self) -> Vec<usize> {
        self.slots.occupied_slots()
    }

    pub fn occupied_slots_iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.slots.occupied_slots_iter()
    }

    pub fn physical_index(&self, slot: usize) -> Option<usize> {
        self.slots.physical_index(slot)
    }

    pub fn retired_at_for_slot(&self, slot: usize) -> Option<VersionId> {
        self.physical_index(slot)
            .and_then(|physical| self.retired_at.get(physical).copied().flatten())
    }

    pub fn metadata_history_at(
        &self,
        slot: usize,
    ) -> Option<&SharedColumn<StorageAllocation<K::Meta>>> {
        self.physical_index(slot)
            .and_then(|physical| self.metadata_history.get(physical))
    }

    pub fn metadata_history_at_mut(
        &mut self,
        slot: usize,
    ) -> Option<&mut SharedColumn<StorageAllocation<K::Meta>>> {
        let physical = self.physical_index(slot)?;
        self.metadata_history.get_mut(physical)
    }

    pub fn aspect_versions_at(&self, slot: usize) -> Option<&AspectVersions> {
        self.physical_index(slot)
            .and_then(|physical| self.aspect_versions.get(physical))
    }

    #[cfg(test)]
    pub fn aspect_versions_at_mut(&mut self, slot: usize) -> Option<&mut AspectVersions> {
        let physical = self.physical_index(slot)?;
        self.aspect_versions.get_mut(physical)
    }

    pub fn field_revisions_at(&self, slot: usize) -> Option<&FieldRevisions> {
        self.physical_index(slot)
            .and_then(|physical| self.field_revisions.get(physical))
            .and_then(Option::as_ref)
    }

    /// Admit the column path before borrowing its shallow map header; each
    /// subsequent map edit admits its actual persistent nodes independently.
    pub fn aspect_versions_at_mut_using(
        &mut self,
        slot: usize,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
    ) -> Result<
        Option<&mut AspectVersions>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let Some(physical) = self.physical_index(slot) else {
            return Ok(None);
        };
        self.aspect_versions.carrier_mut_using(physical, request)
    }
    pub fn field_revisions_at_mut_using(
        &mut self,
        slot: usize,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
    ) -> Result<
        Option<&mut FieldRevisions>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let Some(physical) = self.physical_index(slot) else {
            return Ok(None);
        };
        Ok(self
            .field_revisions
            .carrier_mut_using(physical, request)?
            .and_then(Option::as_mut))
    }
    pub fn replace_revision_maps_for_request(
        &mut self,
        slot: usize,
        aspect_versions: AspectVersions,
        field_revisions: Option<FieldRevisions>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<()>, crate::execution::denial_cause::RelationalExecutionDenialCause> {
        let Some(physical) = self.physical_index(slot) else {
            return Ok(None);
        };
        self.aspect_versions
            .set_value_using(physical, aspect_versions, Some(request))?;
        self.field_revisions
            .set_value_using(physical, field_revisions, Some(request))?;
        Ok(Some(()))
    }
    pub fn retained_extra(
        &self,
        slot: usize,
    ) -> Option<crate::storage::substrate::StorageAllocation<K::Extra>> {
        self.extra.retained_value(self.physical_index(slot)?)
    }

    pub fn extra_at(&self, slot: usize) -> Option<&K::Extra> {
        self.physical_index(slot)
            .and_then(|physical| self.extra.get(physical))
    }

    pub fn created_at_for_slot(&self, slot: usize) -> Option<VersionId> {
        self.physical_index(slot)
            .and_then(|physical| self.created_at.get(physical).copied())
    }

    /// Canonical truth/lifecycle/index bytes only. Diagnostics, retention
    /// counters, and allocator bookkeeping have independent lifecycle lanes.
    #[cfg(test)]
    pub fn authoritative_allocation_bytes(&self) -> u64 {
        self.allocation_inventory()
            .expect("test arena allocation inventory fits u64")
            .authoritative_bytes
    }
}

pub type EntityArena = RecordArena<EntityRecordKind>;
pub type RelationArena = RecordArena<RelationRecordKind>;

#[derive(Clone, Copy)]
pub enum PinClass {
    Branch,
    Replay,
}

pub fn lifecycle_counts<'a>(
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
