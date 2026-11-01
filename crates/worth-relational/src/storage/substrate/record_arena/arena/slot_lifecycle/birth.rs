//! A single birth/reuse algorithm installs only fully prepared metadata.
use super::*;
impl<K: RecordKind> RecordArena<K> {
    pub(super) fn write_reserved_slot_using(
        &mut self,
        init: SlotInit<K>,
        slot: usize,
        generation: u32,
        prepare: &mut impl FnMut(K::Meta) -> Result<K::Meta, Cause>,
    ) -> Result<bool, MetadataWriteDenial> {
        if let Some(physical) = self.physical_index(slot) {
            self.install_reused_slot_using(init, physical, generation, prepare)?;
            return Ok(true);
        }
        self.append_reserved_slot_using(init, slot, generation, prepare)?;
        Ok(false)
    }

    fn install_reused_slot_using(
        &mut self,
        init: SlotInit<K>,
        physical: usize,
        generation: u32,
        prepare: &mut impl FnMut(K::Meta) -> Result<K::Meta, Cause>,
    ) -> Result<(), MetadataWriteDenial> {
        let SlotInit {
            partition_id,
            kind_id,
            version_id,
            extra,
        } = init;
        let extra = crate::storage::substrate::StorageAllocation::new(extra);
        let metadata = K::metadata_for_create(kind_id, generation, version_id, extra.clone())
            .ok_or(MetadataWriteDenial::Invalid(
                "record extra cannot construct its native metadata",
            ))?;
        let metadata = prepare(metadata)?;
        let retired = self.prepare_retired_metadata(physical, version_id, prepare)?;
        self.install_retired_metadata(physical, retired);
        self.partition_ids[physical] = partition_id;
        self.generations[physical] = generation;
        self.lifecycle[physical] = RecordLifecycleState::Live;
        self.kind_ids[physical] = Some(kind_id);
        self.metadata_history[physical].push(metadata);
        self.created_at[physical] = version_id;
        self.retired_at[physical] = None;
        self.extra.set_shared(physical, extra);
        self.aspect_versions.set(physical, Default::default());
        self.field_revisions.set(physical, Some(Default::default()));
        self.diagnostics_enrichment
            .set(physical, Default::default());
        self.branch_pins.set(physical, 0);
        self.replay_pins.set(physical, 0);
        self.snapshot_pins.set(physical, 0);
        let logical = self.slots.slots()[physical] as usize;
        self.live_bitset.set(logical, true);
        self.reclaimable_bitset.set(logical, false);
        Ok(())
    }

    fn append_reserved_slot_using(
        &mut self,
        init: SlotInit<K>,
        logical_slot: usize,
        generation: u32,
        prepare: &mut impl FnMut(K::Meta) -> Result<K::Meta, Cause>,
    ) -> Result<(), MetadataWriteDenial> {
        let SlotInit {
            partition_id,
            kind_id,
            version_id,
            extra,
        } = init;
        let extra = crate::storage::substrate::StorageAllocation::new(extra);
        let metadata = K::metadata_for_create(kind_id, generation, version_id, extra.clone())
            .ok_or(MetadataWriteDenial::Invalid(
                "record extra cannot construct its native metadata",
            ))?;
        let metadata = prepare(metadata)?;
        self.slots.insert(logical_slot as u64)?;
        self.partition_ids.push(partition_id);
        self.generations.push(generation);
        self.lifecycle.push(RecordLifecycleState::Live);
        self.kind_ids.push(Some(kind_id));
        self.metadata_history.push(vec![metadata].into());
        self.history_prefixes.push(None);
        self.created_at.push(version_id);
        self.retired_at.push(None);
        self.extra.push_shared(extra);
        self.aspect_versions.push(std::collections::BTreeMap::new());
        self.field_revisions
            .push(Some(std::collections::BTreeMap::new()));
        self.diagnostics_enrichment
            .push(std::collections::BTreeMap::new());
        self.branch_pins.push(0);
        self.replay_pins.push(0);
        self.snapshot_pins.push(0);
        self.live_bitset.set(logical_slot, true);
        self.reclaimable_bitset.set(logical_slot, false);
        Ok(())
    }
}
