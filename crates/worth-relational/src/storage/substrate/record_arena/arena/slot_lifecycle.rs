mod birth;
mod current_slot_data;
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
        self.current_slot_data(slot)
            .ok_or("rematerialization requires an existing slot")?
            .rematerialize_using(id, kind_id, version_id, extra, request, prepare)
    }

    pub(super) fn apply_extra_update_using(
        &mut self,
        slot: usize,
        extra: K::Extra,
        version_id: VersionId,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        prepare: &mut impl FnMut(K::Meta) -> Result<K::Meta, Cause>,
    ) -> Result<Option<()>, Cause> {
        let Some(mut data) = self.current_slot_data(slot) else {
            return Ok(None);
        };
        data.apply_extra_update_using(extra, version_id, request, prepare)
    }

    pub(super) fn retire_using(
        &mut self,
        slot: usize,
        version_id: VersionId,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        prepare: &mut impl FnMut(K::Meta) -> Result<K::Meta, Cause>,
    ) -> Result<Option<()>, Cause> {
        let Some(mut data) = self.current_slot_data(slot) else {
            return Ok(None);
        };
        data.retire_using(version_id, request, prepare)
    }

    fn set_presence_bits_using(
        &mut self,
        slot: usize,
        live: bool,
        reclaimable: bool,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
    ) -> Result<(), Cause> {
        set_slot_presence(
            &mut self.live_bitset,
            &mut self.reclaimable_bitset,
            slot,
            live,
            reclaimable,
            request,
        )
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
        self.current_slot_data(slot)?.reset_slot();
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

pub(super) fn set_slot_presence(
    live_bitset: &mut crate::storage::partition::DenseSlotBitSet,
    reclaimable_bitset: &mut crate::storage::partition::DenseSlotBitSet,
    slot: usize,
    live: bool,
    reclaimable: bool,
    request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
) -> Result<(), Cause> {
    if let Some(request) = request {
        live_bitset.set_for_request(slot, live, request)?;
        reclaimable_bitset.set_for_request(slot, reclaimable, request)?;
    } else {
        live_bitset.set(slot, live);
        reclaimable_bitset.set(slot, reclaimable);
    }
    Ok(())
}
