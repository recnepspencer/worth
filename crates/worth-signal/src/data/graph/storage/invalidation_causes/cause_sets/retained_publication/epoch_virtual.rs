//! Root-admitted metadata replay for one retained epoch draft.
use super::{
    NormalizedCauseSet, PendingCauseSetId, PreparedCauseSlot, RetainedCauseStorePublicationDraft,
    SignalError, Work,
};

impl RetainedCauseStorePublicationDraft {
    pub(crate) fn apply_epoch_virtual_slot(
        &mut self,
        slot: PreparedCauseSlot,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        if self.store.sets.len() != slot.sets || self.store.free_indices.len() != slot.free {
            return Err(SignalError::invalid_input(
                "retained epoch virtual slot changed",
            ));
        }
        if let Some(current) = slot.current.index {
            let index = current.get() as usize - 1;
            if self.store.slot_generations.get(index).copied() != Some(slot.current.generation) {
                return Err(SignalError::invalid_input(
                    "retained epoch generation changed",
                ));
            }
            if slot.empty {
                if !self.store.sets[index].is_empty() {
                    self.remove_references_at(index, work)?;
                    self.replace_set(index, Vec::new(), work)?;
                }
                self.store.occupied_set_count = self
                    .store
                    .occupied_set_count
                    .checked_sub(1)
                    .ok_or_else(|| {
                        SignalError::invalid_input("retained epoch occupancy underflow")
                    })?;
                self.replace_generation(index, slot.current.generation.wrapping_add(1), work)?;
                self.push_free_index(index as u32, work)?;
            }
        } else if !slot.empty {
            let next = slot.next.index.expect("nonempty virtual handle");
            let index = next.get() as usize - 1;
            if index < self.store.sets.len() {
                if self.store.free_indices.last().copied() != Some(index as u32)
                    || self.store.slot_generations[index] != slot.next.generation
                    || !self.store.sets[index].is_empty()
                {
                    return Err(SignalError::invalid_input(
                        "retained epoch free slot changed",
                    ));
                }
                self.pop_free_index(work)?;
            } else if index == self.store.sets.len()
                && slot.next.generation == self.store.generation
            {
                self.push_set(Vec::new(), work)?;
                self.push_generation(self.store.generation, work)?;
            } else {
                return Err(SignalError::invalid_input("retained epoch append changed"));
            }
            self.store.occupied_set_count = self
                .store
                .occupied_set_count
                .checked_add(1)
                .ok_or_else(|| SignalError::internal("retained epoch occupancy overflow"))?;
        }
        Ok(())
    }

    pub(crate) fn finish_epoch_virtual_slot(
        &mut self,
        handle: PendingCauseSetId,
        causes: NormalizedCauseSet,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        if causes.is_empty() {
            return Err(SignalError::invalid_input(
                "empty retained epoch final cause",
            ));
        }
        let index = handle
            .index
            .ok_or_else(|| SignalError::invalid_input("missing retained epoch final slot"))?
            .get() as usize
            - 1;
        if self.store.slot_generations.get(index).copied() != Some(handle.generation) {
            return Err(SignalError::invalid_input(
                "retained epoch final slot changed",
            ));
        }
        let causes = causes.into_vec();
        self.add_references(&causes, work)?;
        self.remove_references_at(index, work)?;
        self.replace_set(index, causes, work)
    }
}
