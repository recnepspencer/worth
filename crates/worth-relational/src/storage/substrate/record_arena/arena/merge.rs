use super::{RecordArena, RecordKind};

impl<K: RecordKind> RecordArena<K> {
    pub(crate) fn contains_live_id(&self, id: &crate::identity::data::RecordId<K::Domain>) -> bool {
        self.get(id).is_some_and(|view| view.is_live())
    }

    /// Adopt the journal-selected payload owners, recording actual path copies.
    pub(crate) fn merge_slots_from_owned<'slots>(
        &mut self,
        overlay: &mut Self,
        touched_slots: impl IntoIterator<Item = &'slots usize>,
    ) -> u64 {
        touched_slots.into_iter().fold(0_u64, |bytes, &slot| {
            bytes.saturating_add(self.move_slot_from_overlay(overlay, slot))
        })
    }

    pub(crate) fn merge_slot_chunks_from_owned<'slots>(
        &mut self,
        overlay: &mut Self,
        touched_slots: impl IntoIterator<Item = &'slots usize>,
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
        let adoption = super::slot_adoption::SlotAdoption::prepare(
            slot, &self.slots, &overlay.slots,
        ).expect(/* Proof: WorkingState::into_partition_commits emits this private journal only from the successful applied-mutation phase. Every touched slot has completed its native arena write; an errored mutation cannot issue PublicationArtifacts. Replay recomputes that journal through mutation rather than decoding one from media. */ "touched overlay slot must be materialized");
        let overlay_physical = adoption.selected_physical();
        let mut copied_bytes = 0_u64;
        if !adoption.appends() {
            let physical = adoption.destination_physical();
            // Pins belong to the live owner, not to the sparse transaction
            // overlay. A new generation may inherit no old obligations.
            if self.generations[physical] != overlay.generations[overlay_physical] {
                debug_assert_eq!(
                    /* Proof: Native publication journals are emitted only after mutating an admitted overlay slot. Generation replacement requires the owner free-slot reservation, which excludes every pin class before reuse; the retained source and destination are exclusively borrowed here. */
                    self.branch_pins[physical],
                    0
                );
                debug_assert_eq!(
                    /* Proof: Native publication journals are emitted only after mutating an admitted overlay slot. Generation replacement requires the owner free-slot reservation, which excludes every pin class before reuse; the retained source and destination are exclusively borrowed here. */
                    self.replay_pins[physical],
                    0
                );
                debug_assert_eq!(
                    /* Proof: Native publication journals are emitted only after mutating an admitted overlay slot. Generation replacement requires the owner free-slot reservation, which excludes every pin class before reuse; the retained source and destination are exclusively borrowed here. */
                    self.snapshot_pins[physical],
                    0
                );
                self.branch_pins.set(physical, 0);
                self.replay_pins.set(physical, 0);
                self.snapshot_pins.set(physical, 0);
            }
        } else {
            copied_bytes = self
                .slots
                .insert(slot as u64)
                .expect(/* Proof: The immediately preceding physical_index(slot) branch selected None from this same destination SlotDirectory under exclusive &mut self. SlotDirectory::insert rejects only an existing key; neither the directory nor slot changes between lookup and insert. */ "new publication slot must be unique");
            self.branch_pins.push(0);
            self.replay_pins.push(0);
            self.snapshot_pins.push(0);
        }

        // Header and dynamic payload owners use the same adoption operation:
        // no discarded prior value and no selected overlay payload is cloned.
        macro_rules! adopt_truth_columns {
            ($($column:ident),+ $(,)?) => { $(
                copied_bytes = copied_bytes.saturating_add(
                    adoption.copy_selected_value(&mut self.$column, &overlay.$column)
                );
            )+ };
        }
        adopt_truth_columns!(
            partition_ids,
            generations,
            lifecycle,
            kind_ids,
            metadata_history,
            history_prefixes,
            created_at,
            retired_at,
            extra,
            aspect_versions,
            field_revisions
        );
        adoption.copy_selected_value(
            &mut self.diagnostics_enrichment,
            &overlay.diagnostics_enrichment,
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
