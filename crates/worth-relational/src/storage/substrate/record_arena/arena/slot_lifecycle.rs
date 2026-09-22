use crate::identity::data::VersionId;
use crate::storage::data::RecordLifecycleState;

use super::{RecordArena, RecordKind, SlotInit};

impl<K: RecordKind> RecordArena<K> {
    pub(crate) fn suspend_materialization(&mut self, slot: usize) -> Result<(), &'static str> {
        let physical = self
            .physical_index(slot)
            .ok_or("materialization suspension requires an existing slot")?;
        if self.lifecycle[physical] != RecordLifecycleState::Live {
            return Err("materialization suspension requires a live record");
        }
        self.lifecycle[physical] = RecordLifecycleState::MaterializationUnavailable;
        self.extra
            .set(physical, K::unavailable_extra(&self.extra[physical]));
        self.metadata_history.set(physical, Default::default());
        self.aspect_versions.set(physical, Default::default());
        self.diagnostics_enrichment
            .set(physical, Default::default());
        self.live_bitset.set(slot, false);
        self.reclaimable_bitset.set(slot, false);
        Ok(())
    }

    pub(crate) fn rematerialize(
        &mut self,
        id: &crate::identity::data::RecordId<K::Domain>,
        kind_id: crate::identity::data::KindId,
        version_id: VersionId,
        extra: K::Extra,
    ) -> Result<(), &'static str> {
        let slot = crate::storage::substrate::slot_of::<K>(id);
        let physical = self
            .physical_index(slot)
            .ok_or("rematerialization requires an existing slot")?;
        if self.lifecycle[physical] != RecordLifecycleState::MaterializationUnavailable {
            return Err("rematerialization requires an unavailable record");
        }
        if self.generations[physical] != crate::storage::substrate::generation_of::<K>(id)
            || self.partition_ids[physical] != crate::storage::substrate::partition_of::<K>(id)
            || self.kind_ids[physical] != Some(kind_id)
        {
            return Err("rematerialization identity or kind does not match retained metadata");
        }
        self.metadata_history[physical].push(K::metadata_for_create(
            kind_id,
            self.generations[physical],
            version_id,
            &extra,
        ));
        self.extra.set(physical, extra);
        self.lifecycle[physical] = RecordLifecycleState::Live;
        self.retired_at[physical] = None;
        self.live_bitset.set(slot, true);
        self.reclaimable_bitset.set(slot, false);
        Ok(())
    }

    pub(crate) fn apply_extra_update(
        &mut self,
        slot: usize,
        extra: K::Extra,
        version_id: VersionId,
    ) {
        let physical = self
            .physical_index(slot)
            .expect("record extra update requires a materialized slot");
        if let Some(current) = self.metadata_history[physical].last_mut() {
            K::retire_metadata(current, version_id);
        }
        let kind_id =
            self.kind_ids[physical].expect("record extra update requires retained kind id");
        self.metadata_history[physical].push(K::metadata_for_create(
            kind_id,
            self.generations[physical],
            version_id,
            &extra,
        ));
        self.extra.set(physical, extra);
    }

    pub(crate) fn retire(&mut self, slot: usize, version_id: VersionId) {
        let physical = self
            .physical_index(slot)
            .expect("record retirement requires a materialized slot");
        self.retired_at[physical] = Some(version_id);
        self.lifecycle[physical] = RecordLifecycleState::DeletedRetained;
        self.live_bitset.set(slot, false);
        self.reclaimable_bitset.set(slot, true);
        if let Some(current) = self.metadata_history[physical].last_mut() {
            K::retire_metadata(current, version_id);
        }
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

    pub(crate) fn write_reserved_slot(
        &mut self,
        init: SlotInit<K>,
        slot: usize,
        generation: u32,
    ) -> Result<bool, &'static str> {
        if let Some(physical) = self.physical_index(slot) {
            self.install_reused_slot(init, physical, generation);
            return Ok(true);
        }
        self.append_reserved_slot(init, slot, generation)?;
        Ok(false)
    }

    fn install_reused_slot(&mut self, init: SlotInit<K>, physical: usize, generation: u32) {
        let SlotInit {
            partition_id,
            kind_id,
            version_id,
            extra,
        } = init;
        if let Some(current) = self.metadata_history[physical].last_mut() {
            K::retire_metadata(current, version_id);
        }
        self.partition_ids[physical] = partition_id;
        self.generations[physical] = generation;
        self.lifecycle[physical] = RecordLifecycleState::Live;
        self.kind_ids[physical] = Some(kind_id);
        self.metadata_history[physical].push(K::metadata_for_create(
            kind_id, generation, version_id, &extra,
        ));
        self.created_at[physical] = version_id;
        self.retired_at[physical] = None;
        self.extra.set(physical, extra);
        self.aspect_versions.set(physical, Default::default());
        self.diagnostics_enrichment
            .set(physical, Default::default());
        self.branch_pins.set(physical, 0);
        self.replay_pins.set(physical, 0);
        self.snapshot_pins.set(physical, 0);
        let logical = self.slots.slots()[physical] as usize;
        self.live_bitset.set(logical, true);
        self.reclaimable_bitset.set(logical, false);
    }

    fn append_reserved_slot(
        &mut self,
        init: SlotInit<K>,
        logical_slot: usize,
        generation: u32,
    ) -> Result<(), &'static str> {
        let SlotInit {
            partition_id,
            kind_id,
            version_id,
            extra,
        } = init;
        self.slots.insert(logical_slot as u64)?;
        self.partition_ids.push(partition_id);
        self.generations.push(generation);
        self.lifecycle.push(RecordLifecycleState::Live);
        self.kind_ids.push(Some(kind_id));
        self.metadata_history.push(
            vec![K::metadata_for_create(
                kind_id, generation, version_id, &extra,
            )]
            .into(),
        );
        self.created_at.push(version_id);
        self.retired_at.push(None);
        self.extra.push(extra);
        self.aspect_versions.push(std::collections::BTreeMap::new());
        self.diagnostics_enrichment
            .push(std::collections::BTreeMap::new());
        self.branch_pins.push(0);
        self.replay_pins.push(0);
        self.snapshot_pins.push(0);
        self.live_bitset.set(logical_slot, true);
        self.reclaimable_bitset.set(logical_slot, false);
        Ok(())
    }

    pub(crate) fn reset_slot(&mut self, slot: usize) {
        let physical = self
            .physical_index(slot)
            .expect("record reset requires a materialized slot");
        self.kind_ids[physical] = None;
        self.extra.set(physical, K::empty_extra());
        self.aspect_versions.set(physical, Default::default());
        self.diagnostics_enrichment
            .set(physical, Default::default());
        self.branch_pins.set(physical, 0);
        self.replay_pins.set(physical, 0);
        self.snapshot_pins.set(physical, 0);
        self.retired_at[physical] = None;
    }

    pub(crate) fn set_lifecycle_for_slot(
        &mut self,
        slot: usize,
        lifecycle: RecordLifecycleState,
    ) -> bool {
        let Some(physical) = self.physical_index(slot) else {
            return false;
        };
        self.lifecycle[physical] = lifecycle;
        true
    }
}
