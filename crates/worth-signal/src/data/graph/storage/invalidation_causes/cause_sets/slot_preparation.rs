//! Slot outcomes in the exact release/replacement order of one publication.
use super::{CanonicalCauseSetStore, NormalizedCauseSet, PendingCauseSetId};
use crate::data::error::SignalError;
use crate::logic::evaluation::EvaluationWork;
use std::num::NonZeroU32;
#[cfg(test)]
mod tests;
mod visited_slots;
use visited_slots::VisitedCauseSlots;

#[derive(Debug)]
pub(crate) struct PreparedCauseSlot {
    pub(super) current: PendingCauseSetId,
    pub(super) next: PendingCauseSetId,
    pub(super) sets: usize,
    pub(super) free: usize,
    pub(super) empty: bool,
}

impl PreparedCauseSlot {
    pub(crate) fn handle(&self) -> PendingCauseSetId {
        self.next
    }
    pub(crate) fn current(&self) -> PendingCauseSetId {
        self.current
    }
}

/// Borrows real free-slot metadata and overlays only this packet's releases.
/// It never clones or scans the inherited free list or cause payloads.
pub(crate) struct CauseSlotPreparation<'a> {
    store: &'a CanonicalCauseSetStore,
    inherited_free: usize,
    released: smallvec::SmallVec<[PendingCauseSetId; 1]>,
    sets: usize,
    claimed: VisitedCauseSlots,
    allocated: VisitedCauseSlots,
}

impl CanonicalCauseSetStore {
    pub(crate) fn prepare_cause_slots(&self) -> Result<CauseSlotPreparation<'_>, SignalError> {
        if self.slot_generations.len() != self.sets.len() {
            return Err(SignalError::invalid_input(
                "cause slot preparation requires complete metadata",
            ));
        }
        Ok(CauseSlotPreparation {
            store: self,
            inherited_free: self.free_indices.len(),
            released: Default::default(),
            sets: self.sets.len(),
            claimed: Default::default(),
            allocated: Default::default(),
        })
    }

    pub(crate) fn publish_prepared_cause_slot(
        &mut self,
        slot: PreparedCauseSlot,
        causes: NormalizedCauseSet,
    ) -> Result<PendingCauseSetId, SignalError> {
        if self.sets.len() != slot.sets
            || self.free_indices.len() != slot.free
            || causes.is_empty() != slot.empty
        {
            return Err(SignalError::invalid_input(
                "prepared cause slot storage changed",
            ));
        }
        if let Some(index) = slot.current.index {
            self.get(slot.current)?;
            if slot.empty {
                self.release(slot.current)?;
            } else {
                let index = index.get() as usize - 1;
                let causes = causes.into_vec();
                self.add_output_commit_references_from(&causes);
                self.remove_output_commit_references_at(index);
                self.sets.replace_discard(index, causes);
            }
        } else if !slot.empty {
            let index = slot.next.index.expect("nonempty prepared handle").get() as usize - 1;
            if index < self.sets.len() {
                if self.free_indices.last().copied() != Some(index as u32)
                    || self.slot_generations[index] != slot.next.generation
                    || !self.sets[index].is_empty()
                {
                    return Err(SignalError::invalid_input(
                        "prepared cause free slot changed",
                    ));
                }
            } else if index != self.sets.len() || slot.next.generation != self.generation {
                return Err(SignalError::invalid_input("prepared cause append changed"));
            }
            let causes = causes.into_vec();
            self.add_output_commit_references_from(&causes);
            if index < self.sets.len() {
                self.free_indices.pop_back();
                self.sets.replace_discard(index, causes);
            } else {
                self.sets.push_back(causes);
                self.slot_generations.push_back(self.generation);
            }
            self.occupied_set_count += 1;
        }
        Ok(slot.next)
    }
}

impl CauseSlotPreparation<'_> {
    /// Account for the producer's earlier clean transition, without writing it.
    pub(crate) fn release(
        &mut self,
        current: PendingCauseSetId,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        let Some(index) = current.index else {
            return Ok(());
        };
        self.claim(current, work)?;
        work.reserve(
            self.released
                .len()
                .checked_add(1)
                .and_then(|n| n.checked_mul(std::mem::size_of::<PendingCauseSetId>() + 1)),
        )?;
        self.released.push(PendingCauseSetId {
            index: Some(index),
            generation: current.generation.wrapping_add(1),
        });
        Ok(())
    }

    pub(crate) fn replacement(
        &mut self,
        current: PendingCauseSetId,
        empty: bool,
        work: &mut EvaluationWork<'_>,
    ) -> Result<PreparedCauseSlot, SignalError> {
        work.reserve(Some(
            self.store.sets.lookup_steps()
                + self.store.slot_generations.lookup_steps()
                + self.store.free_indices.lookup_steps()
                + 8,
        ))?;
        let free = self
            .inherited_free
            .checked_add(self.released.len())
            .ok_or_else(|| SignalError::internal("cause free-slot count overflow"))?;
        let sets = self.sets;
        let next = if empty {
            self.release(current, work)?;
            PendingCauseSetId::EMPTY
        } else if current.index.is_some() {
            self.claim(current, work)?;
            current
        } else if let Some(free) = self.released.pop() {
            free
        } else if self.inherited_free > 0 {
            self.inherited_free -= 1;
            let index = self.store.free_indices[self.inherited_free] as usize;
            if !self.store.sets.get(index).is_some_and(Vec::is_empty) {
                return Err(SignalError::invalid_input("cause free slot is not vacant"));
            }
            PendingCauseSetId {
                index: NonZeroU32::new(index as u32 + 1),
                generation: self.store.slot_generations[index],
            }
        } else {
            let index = u32::try_from(self.sets)
                .ok()
                .and_then(|n| n.checked_add(1))
                .and_then(NonZeroU32::new)
                .ok_or_else(|| SignalError::internal("cause slot handle capacity exhausted"))?;
            self.sets += 1;
            PendingCauseSetId {
                index: Some(index),
                generation: self.store.generation,
            }
        };
        if !empty && current.index.is_none() {
            work.reserve(Some(
                crate::data::retained_storage::ordered_lookup_steps(self.allocated.len()) * 32,
            ))?;
            if !self
                .allocated
                .insert(next.index.expect("allocated handle").get())
            {
                return Err(SignalError::invalid_input(
                    "duplicate prepared cause allocation",
                ));
            }
        }
        Ok(PreparedCauseSlot {
            current,
            next,
            sets,
            free,
            empty,
        })
    }

    fn claim(
        &mut self,
        current: PendingCauseSetId,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        work.reserve(Some(
            self.store.sets.lookup_steps()
                + self.store.slot_generations.lookup_steps()
                + crate::data::retained_storage::ordered_lookup_steps(self.claimed.len()) * 32,
        ))?;
        if self.store.get(current)?.is_empty()
            || !self
                .claimed
                .insert(current.index.expect("occupied handle").get())
        {
            return Err(SignalError::invalid_input(
                "duplicate or vacant prepared cause owner",
            ));
        }
        Ok(())
    }
}
