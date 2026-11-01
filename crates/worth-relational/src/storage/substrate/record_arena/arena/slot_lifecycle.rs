mod birth;
mod metadata;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::identity::data::VersionId;
use crate::storage::data::RecordLifecycleState;
pub(crate) use metadata::MetadataWriteDenial;

use super::{RecordArena, RecordKind, SlotInit};

impl<K: RecordKind> RecordArena<K> {
    #[cfg(test)]
    pub(crate) fn suspend_materialization(&mut self, slot: usize) -> Result<(), &'static str> {
        let physical = self
            .physical_index(slot)
            .ok_or("materialization suspension requires an existing slot")?;
        let extra = K::unavailable_extra(&self.extra[physical]);
        self.suspend_materialization_with_extra(slot, extra)
    }

    pub(crate) fn suspend_materialization_with_extra_for_request(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        slot: usize,
        extra: K::Extra,
    ) -> Result<(), MetadataWriteDenial> {
        self.suspend_using(slot, extra, Some(request))
    }
    /// Existing ordinary fixture/maintenance construction is unqualified.
    #[cfg(test)]
    pub(crate) fn suspend_materialization_with_extra(
        &mut self,
        slot: usize,
        extra: K::Extra,
    ) -> Result<(), &'static str> {
        self.suspend_using(slot, extra, None)
            .map_err(metadata::ordinary_denial)
    }
    fn suspend_using(
        &mut self,
        slot: usize,
        extra: K::Extra,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
    ) -> Result<(), MetadataWriteDenial> {
        let physical = self
            .physical_index(slot)
            .ok_or("materialization suspension requires an existing slot")?;
        if self.lifecycle[physical] != RecordLifecycleState::Live {
            return Err(MetadataWriteDenial::Invalid(
                "materialization suspension requires a live record",
            ));
        }
        self.lifecycle.set_value_using(
            physical,
            RecordLifecycleState::MaterializationUnavailable,
            request,
        )?;
        self.extra.set_value_using(physical, extra, request)?;
        self.metadata_history
            .set_value_using(physical, Default::default(), request)?;
        self.history_prefixes
            .set_value_using(physical, None, request)?;
        self.aspect_versions
            .set_value_using(physical, Default::default(), request)?;
        self.field_revisions
            .set_value_using(physical, None, request)?;
        self.diagnostics_enrichment
            .set_value_using(physical, Default::default(), request)?;
        self.set_presence_bits_using(slot, false, false, request)?;
        Ok(())
    }

    pub(super) fn rematerialize_using(
        &mut self,
        id: &crate::identity::data::RecordId<K::Domain>,
        kind_id: crate::identity::data::KindId,
        version_id: VersionId,
        extra: K::Extra,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        prepare: &mut impl FnMut(K::Meta) -> Result<K::Meta, Cause>,
    ) -> Result<(), MetadataWriteDenial> {
        let slot = crate::storage::substrate::slot_of::<K>(id);
        let physical = self
            .physical_index(slot)
            .ok_or("rematerialization requires an existing slot")?;
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
        slot: usize,
        extra: K::Extra,
        version_id: VersionId,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        prepare: &mut impl FnMut(K::Meta) -> Result<K::Meta, Cause>,
    ) -> Result<Option<()>, Cause> {
        let Some(physical) = self.physical_index(slot) else {
            return Ok(None);
        };
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
        slot: usize,
        version_id: VersionId,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        prepare: &mut impl FnMut(K::Meta) -> Result<K::Meta, Cause>,
    ) -> Result<Option<()>, Cause> {
        let Some(physical) = self.physical_index(slot) else {
            return Ok(None);
        };
        let retired = self.prepare_retired_metadata(physical, version_id, prepare)?;
        self.retired_at
            .set_value_using(physical, Some(version_id), request)?;
        self.lifecycle
            .set_value_using(physical, RecordLifecycleState::DeletedRetained, request)?;
        self.set_presence_bits_using(slot, false, true, request)?;
        self.install_retired_metadata(physical, retired, request)?;
        Ok(Some(()))
    }

    fn set_presence_bits_using(
        &mut self,
        slot: usize,
        live: bool,
        reclaimable: bool,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
    ) -> Result<(), Cause> {
        if let Some(request) = request {
            self.live_bitset.set_for_request(slot, live, request)?;
            self.reclaimable_bitset
                .set_for_request(slot, reclaimable, request)?;
        } else {
            self.live_bitset.set(slot, live);
            self.reclaimable_bitset.set(slot, reclaimable);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn push_slot(&mut self, init: SlotInit<K>) -> (usize, u32, bool) {
        self.push_slot_with_generation(init, |_, generation| generation.saturating_add(1))
    }

    #[cfg(test)]
    pub(crate) fn push_slot_with_generation(
        &mut self,
        init: SlotInit<K>,
        issue_generation: impl FnOnce(usize, u32) -> u32,
    ) -> (usize, u32, bool) {
        let slot = self
            .lifecycle
            .iter()
            .position(|state| *state == RecordLifecycleState::Reusable)
            .map(|physical| self.slots.slots()[physical] as usize)
            .unwrap_or_else(|| {
                self.occupied_slots()
                    .last()
                    .copied()
                    .map_or(0, |slot| slot.saturating_add(1))
            });
        let observed = self
            .physical_index(slot)
            .and_then(|physical| self.generations.get(physical).copied())
            .unwrap_or(0);
        let generation = issue_generation(slot, observed);
        let reused = self
            .write_reserved_slot(init, slot, generation)
            .expect("arena-owned free-list slot must be structurally admissible");
        (slot, generation, reused)
    }

    pub(crate) fn reset_slot(&mut self, slot: usize) -> Option<()> {
        let physical = self.physical_index(slot)?;
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
        Some(())
    }

    pub(crate) fn set_lifecycle_for_slot(
        &mut self,
        slot: usize,
        lifecycle: RecordLifecycleState,
    ) -> bool {
        let Some(physical) = self.physical_index(slot) else {
            return false;
        };
        self.lifecycle.set(physical, lifecycle);
        true
    }
    pub(crate) fn set_lifecycle_for_slot_for_request(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        slot: usize,
        lifecycle: RecordLifecycleState,
    ) -> Result<bool, Cause> {
        let Some(physical) = self.physical_index(slot) else {
            return Ok(false);
        };
        self.lifecycle
            .set_for_request(physical, lifecycle, request)?;
        Ok(true)
    }
}
