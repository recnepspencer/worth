use crate::storage::substrate::SharedColumn;
use std::collections::BTreeMap;

use super::{RecordArena, RecordKind};

impl<K: RecordKind> RecordArena<K> {
    pub(crate) fn sparse_clone_slots_for_overlay(
        &self,
        touched_slots: &std::collections::BTreeSet<usize>,
    ) -> Self
    where
        K::Extra: Clone,
        K::Meta: Clone,
    {
        let slot_count = self.slot_count();
        let mut clone = self.sparse_overlay_shell(slot_count);

        for &slot in touched_slots {
            let Some(physical) = self.physical_index(slot) else {
                continue;
            };
            clone
                .metadata_history
                .copy_value_from(physical, &self.metadata_history, physical);
            clone.extra.copy_value_from(physical, &self.extra, physical);
            clone
                .aspect_versions
                .copy_value_from(physical, &self.aspect_versions, physical);
            clone
                .field_revisions
                .copy_value_from(physical, &self.field_revisions, physical);
            clone.diagnostics_enrichment.copy_value_from(
                physical,
                &self.diagnostics_enrichment,
                physical,
            );
        }

        clone
    }

    pub(crate) fn sparse_shape_clone_for_overlay(&self) -> Self
    where
        K::Extra: Clone,
        K::Meta: Clone,
    {
        self.sparse_overlay_shell(self.slot_count())
    }

    fn sparse_overlay_shell(&self, slot_count: usize) -> Self
    where
        K::Extra: Clone,
        K::Meta: Clone,
    {
        let mut clone = Self::with_capacity(slot_count);
        clone.slots.clone_from(&self.slots);
        clone.partition_ids.clone_from(&self.partition_ids);
        clone.generations.clone_from(&self.generations);
        clone.lifecycle.clone_from(&self.lifecycle);
        clone.kind_ids.clone_from(&self.kind_ids);
        clone.created_at.clone_from(&self.created_at);
        clone.retired_at.clone_from(&self.retired_at);
        clone.branch_pins.clone_from(&self.branch_pins);
        clone.replay_pins.clone_from(&self.replay_pins);
        clone.snapshot_pins.clone_from(&self.snapshot_pins);
        clone.live_bitset = self.live_bitset.clone();
        clone.reclaimable_bitset = self.reclaimable_bitset.clone();

        clone.metadata_history = SharedColumn::with_default(slot_count, SharedColumn::default());
        clone.extra = SharedColumn::with_default(slot_count, K::empty_extra());
        clone.aspect_versions = SharedColumn::with_default(slot_count, BTreeMap::new());
        clone.field_revisions = SharedColumn::with_default(slot_count, None);
        clone.diagnostics_enrichment = SharedColumn::with_default(slot_count, BTreeMap::new());

        clone
    }
}
