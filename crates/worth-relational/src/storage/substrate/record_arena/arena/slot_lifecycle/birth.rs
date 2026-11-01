//! A single birth/reuse algorithm installs only fully prepared metadata.
use super::*;
impl<K: RecordKind> RecordArena<K> {
    pub(super) fn write_reserved_slot_using(
        &mut self,
        init: SlotInit<K>,
        slot: usize,
        generation: u32,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        prepare: &mut impl FnMut(K::Meta) -> Result<K::Meta, Cause>,
    ) -> Result<bool, MetadataWriteDenial> {
        if let Some(physical) = self.physical_index(slot) {
            self.install_reused_slot_using(init, physical, generation, request, prepare)?;
            return Ok(true);
        }
        self.append_reserved_slot_using(init, slot, generation, request, prepare)?;
        Ok(false)
    }

    fn install_reused_slot_using(
        &mut self,
        init: SlotInit<K>,
        physical: usize,
        generation: u32,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
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
        self.install_retired_metadata(physical, retired, request)?;
        self.partition_ids
            .set_value_using(physical, partition_id, request)?;
        self.generations
            .set_value_using(physical, generation, request)?;
        self.lifecycle
            .set_value_using(physical, RecordLifecycleState::Live, request)?;
        self.kind_ids
            .set_value_using(physical, Some(kind_id), request)?;
        self.metadata_history
            .carrier_mut_using(physical, request)?
            .expect("existing history")
            .push_value_using(metadata, request)?;
        self.created_at
            .set_value_using(physical, version_id, request)?;
        self.retired_at.set_value_using(physical, None, request)?;
        self.extra.set_shared_using(physical, extra, request)?;
        self.aspect_versions
            .set_value_using(physical, Default::default(), request)?;
        self.field_revisions
            .set_value_using(physical, Some(Default::default()), request)?;
        self.diagnostics_enrichment
            .set_value_using(physical, Default::default(), request)?;
        self.branch_pins.set_using(physical, 0, request)?;
        self.replay_pins.set_using(physical, 0, request)?;
        self.snapshot_pins.set_using(physical, 0, request)?;
        let logical = self.slots.slots()[physical] as usize;
        self.set_presence_bits_using(logical, true, false, request)?;
        Ok(())
    }

    fn append_reserved_slot_using(
        &mut self,
        init: SlotInit<K>,
        logical_slot: usize,
        generation: u32,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
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
        if let Some(request) = request {
            self.slots
                .insert_for_request(logical_slot as u64, request)?;
        } else {
            self.slots.insert(logical_slot as u64)?;
        }
        self.partition_ids.push_value_using(partition_id, request)?;
        self.generations.push_value_using(generation, request)?;
        self.lifecycle
            .push_value_using(RecordLifecycleState::Live, request)?;
        self.kind_ids.push_value_using(Some(kind_id), request)?;
        let input_memory = crate::storage::substrate::node_memory::reserve(
            request,
            crate::storage::substrate::node_memory::array::<K::Meta>(1),
        )?;
        let values = vec![metadata];
        let history = if let Some(request) = request {
            crate::storage::substrate::SharedColumn::from_dense_values_for_request(values, request)?
        } else {
            values.into()
        };
        drop(input_memory);
        self.metadata_history.push_value_using(history, request)?;
        self.history_prefixes.push_value_using(None, request)?;
        self.created_at.push_value_using(version_id, request)?;
        self.retired_at.push_value_using(None, request)?;
        if let Some(request) = request {
            self.extra.push_shared_for_request(extra, request)?;
        } else {
            self.extra.push_shared(extra);
        }
        self.aspect_versions
            .push_value_using(Default::default(), request)?;
        self.field_revisions
            .push_value_using(Some(Default::default()), request)?;
        self.diagnostics_enrichment
            .push_value_using(std::collections::BTreeMap::new(), request)?;
        self.branch_pins.push(0);
        self.replay_pins.push(0);
        self.snapshot_pins.push(0);
        self.set_presence_bits_using(logical_slot, true, false, request)?;
        Ok(())
    }
}
