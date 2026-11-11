//! Existing-slot current/suffix mutation cannot access omitted history prefixes.
use super::super::{
    AspectVersions, DenseSlotBitSet, FieldRevisions, KindId, PartitionId, RecordPinCounts,
    SharedColumn, StorageAllocation, Symbol,
};
use super::*;
use std::collections::BTreeMap;
mod generation_reuse;
mod metadata_retirement;

pub(super) struct CurrentSlotData<'a, K: RecordKind> {
    physical: usize,
    logical: usize,
    partition_ids: &'a mut SharedColumn<PartitionId>,
    generations: &'a mut SharedColumn<u32>,
    lifecycle: &'a mut SharedColumn<RecordLifecycleState>,
    kind_ids: &'a mut SharedColumn<Option<KindId>>,
    metadata_history: &'a mut SharedColumn<SharedColumn<StorageAllocation<K::Meta>>>,
    created_at: &'a mut SharedColumn<VersionId>,
    retired_at: &'a mut SharedColumn<Option<VersionId>>,
    extra: &'a mut SharedColumn<StorageAllocation<K::Extra>>,
    aspect_versions: &'a mut SharedColumn<AspectVersions>,
    field_revisions: &'a mut SharedColumn<Option<FieldRevisions>>,
    diagnostics_enrichment: &'a mut SharedColumn<StorageAllocation<BTreeMap<Symbol, String>>>,
    branch_pins: &'a mut RecordPinCounts,
    replay_pins: &'a mut RecordPinCounts,
    snapshot_pins: &'a mut RecordPinCounts,
    live_bitset: &'a mut DenseSlotBitSet,
    reclaimable_bitset: &'a mut DenseSlotBitSet,
}

impl<K: RecordKind> RecordArena<K> {
    pub(super) fn current_slot_data(&mut self, logical: usize) -> Option<CurrentSlotData<'_, K>> {
        let physical = self.physical_index(logical)?;
        Some(CurrentSlotData {
            physical,
            logical,
            partition_ids: &mut self.partition_ids,
            generations: &mut self.generations,
            lifecycle: &mut self.lifecycle,
            kind_ids: &mut self.kind_ids,
            metadata_history: &mut self.metadata_history,
            created_at: &mut self.created_at,
            retired_at: &mut self.retired_at,
            extra: &mut self.extra,
            aspect_versions: &mut self.aspect_versions,
            field_revisions: &mut self.field_revisions,
            diagnostics_enrichment: &mut self.diagnostics_enrichment,
            branch_pins: &mut self.branch_pins,
            replay_pins: &mut self.replay_pins,
            snapshot_pins: &mut self.snapshot_pins,
            live_bitset: &mut self.live_bitset,
            reclaimable_bitset: &mut self.reclaimable_bitset,
        })
    }
}

impl<K: RecordKind> CurrentSlotData<'_, K> {
    pub(super) fn rematerialize_using(
        &mut self,
        id: &crate::identity::data::RecordId<K::Domain>,
        kind_id: crate::identity::data::KindId,
        version_id: VersionId,
        extra: K::Extra,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        prepare: &mut impl FnMut(K::Meta) -> Result<K::Meta, Cause>,
    ) -> Result<(), MetadataWriteDenial> {
        let slot = self.logical;
        let physical = self.physical;
        if self.lifecycle[physical] != RecordLifecycleState::MaterializationUnavailable {
            return Err(MetadataWriteDenial::Invalid(
                "rematerialization requires an unavailable record",
            ));
        }
        if self.generations[physical] != crate::storage::substrate::generation_of::<K>(id)
            || self.partition_ids[physical] != crate::storage::substrate::partition_of::<K>(id)
            || self.kind_ids[physical] != Some(kind_id)
        {
            return Err(MetadataWriteDenial::Invalid(
                "rematerialization identity or kind does not match retained metadata",
            ));
        }
        let extra = crate::storage::substrate::StorageAllocation::new(extra);
        let metadata = K::metadata_for_create(
            kind_id,
            self.generations[physical],
            version_id,
            extra.clone(),
        )
        .ok_or(MetadataWriteDenial::Invalid(
            "record extra cannot construct its native metadata",
        ))?;
        let metadata = prepare(metadata)?;
        self.metadata_history
            .carrier_mut_using(physical, request)?
            .expect("existing metadata history")
            .push_value_using(metadata, request)?;
        self.extra.set_shared_using(physical, extra, request)?;
        self.field_revisions
            .set_value_using(physical, Some(Default::default()), request)?;
        self.lifecycle
            .set_value_using(physical, RecordLifecycleState::Live, request)?;
        self.retired_at.set_value_using(physical, None, request)?;
        self.set_presence_bits_using(slot, true, false, request)?;
        Ok(())
    }

    pub(super) fn apply_extra_update_using(
        &mut self,
        extra: K::Extra,
        version_id: VersionId,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        prepare: &mut impl FnMut(K::Meta) -> Result<K::Meta, Cause>,
    ) -> Result<Option<()>, Cause> {
        let physical = self.physical;
        let Some(kind_id) = self.kind_ids[physical] else {
            return Ok(None);
        };
        let extra = crate::storage::substrate::StorageAllocation::new(extra);
        let Some(metadata) = K::metadata_for_create(
            kind_id,
            self.generations[physical],
            version_id,
            extra.clone(),
        ) else {
            return Ok(None);
        };
        let metadata = prepare(metadata)?;
        let retired = self.prepare_retired_metadata(physical, version_id, prepare)?;
        self.install_retired_metadata(physical, retired, request)?;
        self.metadata_history
            .carrier_mut_using(physical, request)?
            .expect("existing metadata history")
            .push_value_using(metadata, request)?;
        self.extra.set_shared_using(physical, extra, request)?;
        Ok(Some(()))
    }

    pub(super) fn retire_using(
        &mut self,
        version_id: VersionId,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        prepare: &mut impl FnMut(K::Meta) -> Result<K::Meta, Cause>,
    ) -> Result<Option<()>, Cause> {
        let slot = self.logical;
        let physical = self.physical;
        let retired = self.prepare_retired_metadata(physical, version_id, prepare)?;
        self.retired_at
            .set_value_using(physical, Some(version_id), request)?;
        self.lifecycle
            .set_value_using(physical, RecordLifecycleState::DeletedRetained, request)?;
        self.set_presence_bits_using(slot, false, true, request)?;
        self.install_retired_metadata(physical, retired, request)?;
        Ok(Some(()))
    }

    pub(super) fn reset_slot(&mut self) {
        let physical = self.physical;
        self.kind_ids[physical] = None;
        self.extra.set(physical, K::empty_extra());
        self.aspect_versions.set(physical, Default::default());
        self.field_revisions.set(physical, None);
        self.diagnostics_enrichment
            .set(physical, Default::default());
        self.branch_pins.set(physical, 0);
        self.replay_pins.set(physical, 0);
        self.snapshot_pins.set(physical, 0);
        self.retired_at[physical] = None;
    }

    fn set_presence_bits_using(
        &mut self,
        slot: usize,
        live: bool,
        reclaimable: bool,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
    ) -> Result<(), Cause> {
        super::set_slot_presence(
            self.live_bitset,
            self.reclaimable_bitset,
            slot,
            live,
            reclaimable,
            request,
        )
    }
}
