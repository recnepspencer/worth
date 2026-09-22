use std::collections::BTreeSet;

use super::{RecordArena, RecordKind};

impl<K: RecordKind> RecordArena<K> {
    pub(crate) fn contains_live_id(&self, id: &crate::identity::data::RecordId<K::Domain>) -> bool {
        self.get(id).is_some_and(|view| view.is_live())
    }

    /// Adopt the journal-selected payload owners, recording actual path copies.
    pub(crate) fn merge_slots_from_owned(
        &mut self,
        overlay: &mut Self,
        touched_slots: &BTreeSet<usize>,
    ) -> u64 {
        touched_slots.iter().fold(0_u64, |bytes, &slot| {
            bytes.saturating_add(self.move_slot_from_overlay(overlay, slot))
        })
    }

    pub(crate) fn merge_slot_chunks_from_owned(
        &mut self,
        overlay: &mut Self,
        touched_slots: &BTreeSet<usize>,
        chunk_width: usize,
    ) -> usize {
        let chunk_width = chunk_width.max(1);
        let mut chunk_count = 0;
        let mut current_chunk = None;
        for &slot in touched_slots {
            let chunk_index = slot / chunk_width;
            if current_chunk != Some(chunk_index) {
                current_chunk = Some(chunk_index);
                chunk_count += 1;
            }
            self.move_slot_from_overlay(overlay, slot);
        }
        chunk_count
    }

    fn move_slot_from_overlay(&mut self, overlay: &Self, slot: usize) -> u64 {
        let overlay_physical = overlay
            .physical_index(slot)
            .expect("touched overlay slot must be materialized");
        let mut copied_bytes = 0_u64;
        let physical = if let Some(physical) = self.physical_index(slot) {
            // Pins belong to the live owner, not to the sparse transaction
            // overlay. A new generation may inherit no old obligations.
            if self.generations[physical] != overlay.generations[overlay_physical] {
                debug_assert_eq!(self.branch_pins[physical], 0);
                debug_assert_eq!(self.replay_pins[physical], 0);
                debug_assert_eq!(self.snapshot_pins[physical], 0);
                self.branch_pins.set(physical, 0);
                self.replay_pins.set(physical, 0);
                self.snapshot_pins.set(physical, 0);
            }
            physical
        } else {
            let physical = self.slots.len();
            copied_bytes = self
                .slots
                .insert(slot as u64)
                .expect("new publication slot must be unique");
            self.branch_pins.push(0);
            self.replay_pins.push(0);
            self.snapshot_pins.push(0);
            physical
        };

        // Header and dynamic payload owners use the same adoption operation:
        // no discarded prior value and no selected overlay payload is cloned.
        macro_rules! adopt_truth_columns {
            ($($column:ident),+ $(,)?) => { $(
                copied_bytes = copied_bytes.saturating_add(
                    self.$column.copy_value_from(physical, &overlay.$column, overlay_physical)
                );
            )+ };
        }
        adopt_truth_columns!(
            partition_ids,
            generations,
            lifecycle,
            kind_ids,
            metadata_history,
            created_at,
            retired_at,
            extra,
            aspect_versions
        );
        self.diagnostics_enrichment.copy_value_from(
            physical,
            &overlay.diagnostics_enrichment,
            overlay_physical,
        );
        copied_bytes = copied_bytes.saturating_add(self.live_bitset.set(
            slot,
            overlay.live_bitset.count_ones_in_range(slot, slot + 1) == 1,
        ));
        copied_bytes.saturating_add(
            self.reclaimable_bitset.set(
                slot,
                overlay
                    .reclaimable_bitset
                    .count_ones_in_range(slot, slot + 1)
                    == 1,
            ),
        )
    }
}
