//! Select a retained version in its original slot; this never allocates an identity.
use super::*;

impl<K: RecordKind> RecordArena<K> {
    pub(crate) fn select_retained_version(
        &mut self,
        id: &RecordId<K::Domain>,
        source: super::SlotView<'_, K>,
        version: VersionId,
    ) -> Result<(), &'static str> {
        let slot = crate::storage::substrate::slot_of::<K>(id);
        if !source.is_live() || source.record_id() != *id {
            return Err("historical selection requires its exact retained live source");
        }
        let birth = source.created_at_version();
        let kind = source.kind_id().ok_or("selected record kind is absent")?;
        let generation = crate::storage::substrate::generation_of::<K>(id);
        let partition = crate::storage::substrate::partition_of::<K>(id);
        let extra = source.extra().clone();
        if let Some(physical) = self.physical_index(slot) {
            if self.generations[physical] != generation
                || self.partition_ids[physical] != partition
                || self.kind_ids[physical] != Some(kind)
                || self.created_at[physical] != birth
                || self.lifecycle[physical] == RecordLifecycleState::Reusable
            {
                return Err("historical selection cannot replace another slot identity");
            }
            self.apply_extra_update(slot, extra, version)
                .ok_or("selected record cannot issue fresh native metadata")?;
            if self.lifecycle[physical] == RecordLifecycleState::MaterializationUnavailable {
                self.field_revisions.set(physical, Some(Default::default()));
            }
            self.lifecycle.set(physical, RecordLifecycleState::Live);
            self.retired_at.set(physical, None);
            self.live_bitset.set(slot, true);
            self.reclaimable_bitset.set(slot, false);
        } else {
            // A sibling branch may never have materialized this globally issued
            // slot. The retained source supplies its original identity and birth.
            self.write_reserved_slot(
                SlotInit {
                    partition_id: partition,
                    kind_id: kind,
                    version_id: version,
                    extra,
                },
                slot,
                generation,
            )?;
            let physical = self
                .physical_index(slot)
                .ok_or("selected slot disappeared")?;
            self.created_at.set(physical, birth);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
