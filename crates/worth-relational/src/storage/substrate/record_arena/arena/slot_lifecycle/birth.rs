//! A single birth/reuse algorithm installs only fully prepared metadata.
use super::*;
impl<K: RecordKind> RecordArena<K> {
    pub(in crate::storage::substrate::record_arena::arena) fn write_reserved_slot_using(
        &mut self,
        init: SlotInit<K>,
        slot: usize,
        generation: u32,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        prepare: &mut impl FnMut(K::Meta) -> Result<K::Meta, Cause>,
    ) -> Result<bool, MetadataWriteDenial> {
        if let Some(mut data) = self.current_slot_data(slot) {
            data.install_reused_slot_using(init, generation, request, prepare)?;
            return Ok(true);
        }
        self.append_reserved_slot_using(init, slot, generation, request, prepare)?;
        Ok(false)
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
