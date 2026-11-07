//! Reuse an existing physical row while its omitted prefix remains inaccessible.
use super::*;
impl<K: RecordKind> CurrentSlotData<'_, K> {
    pub(in crate::storage::substrate::record_arena::arena::slot_lifecycle) fn install_reused_slot_using(
        &mut self,
        init: SlotInit<K>,
        generation: u32,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        prepare: &mut impl FnMut(K::Meta) -> Result<K::Meta, Cause>,
    ) -> Result<(), MetadataWriteDenial> {
        let physical = self.physical;
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
        let logical = self.logical;
        self.set_presence_bits_using(logical, true, false, request)?;
        Ok(())
    }
}
