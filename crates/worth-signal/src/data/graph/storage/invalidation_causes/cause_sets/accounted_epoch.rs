//! Charge-aware ordinary epoch drafts. The live store is installed only after
//! every cause, reference and output root has been prepared successfully.

use super::retained_publication::{
    accounted_map, accounted_vector, map_map_mutation, map_vector_mutation,
};
use super::{CanonicalCauseSetStore, NormalizedCauseSet, PendingCauseSetId, PreparedCauseSlot};
use crate::data::error::SignalError;
use crate::data::proof::invalidation::binding::ResolvedDependencyCause;
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;
use crate::data::retained_storage::RetainedStoragePreparation as Work;

impl CanonicalCauseSetStore {
    /// Replay only slot metadata in a private epoch draft. Empty set payloads
    /// may temporarily represent held slots until final causes are installed.
    pub(crate) fn apply_epoch_virtual_slot(
        &mut self,
        slot: PreparedCauseSlot,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        if self.sets.len() != slot.sets || self.free_indices.len() != slot.free {
            return Err(SignalError::invalid_input(
                "epoch virtual slot storage changed",
            ));
        }
        if let Some(current) = slot.current.index {
            let index = current.get() as usize - 1;
            if self.slot_generations.get(index).copied() != Some(slot.current.generation) {
                return Err(SignalError::invalid_input(
                    "epoch virtual slot generation changed",
                ));
            }
            if slot.empty {
                if !self.sets[index].is_empty() {
                    self.remove_epoch_references_at(index, work)?;
                    accounted_vector(
                        self.sets
                            .edit_with_retained_charge(index, work, |set| *set = Vec::new())
                            .map_err(map_vector_mutation)?,
                    )?;
                }
                self.occupied_set_count = self
                    .occupied_set_count
                    .checked_sub(1)
                    .ok_or_else(|| SignalError::invalid_input("epoch cause occupancy underflow"))?;
                accounted_vector(
                    self.slot_generations
                        .edit_with_retained_charge(index, work, |generation| {
                            *generation = generation.wrapping_add(1)
                        })
                        .map_err(map_vector_mutation)?,
                )?;
                accounted_vector(
                    self.free_indices
                        .push_with_retained_charge(index as u32, work)
                        .map_err(map_vector_mutation)?,
                )?;
            }
        } else if !slot.empty {
            let next = slot.next.index.expect("nonempty virtual handle");
            let index = next.get() as usize - 1;
            if index < self.sets.len() {
                if self.free_indices.last().copied() != Some(index as u32)
                    || self.slot_generations[index] != slot.next.generation
                    || !self.sets[index].is_empty()
                {
                    return Err(SignalError::invalid_input(
                        "epoch virtual free slot changed",
                    ));
                }
                let removed = accounted_vector(
                    self.free_indices
                        .pop_with_retained_charge(work)
                        .map_err(map_vector_mutation)?,
                )?;
                if removed != Some(index as u32) {
                    return Err(SignalError::invalid_input(
                        "epoch virtual free tail changed",
                    ));
                }
            } else if index == self.sets.len() && slot.next.generation == self.generation {
                accounted_vector(
                    self.sets
                        .push_with_retained_charge(Vec::new(), work)
                        .map_err(map_vector_mutation)?,
                )?;
                accounted_vector(
                    self.slot_generations
                        .push_with_retained_charge(self.generation, work)
                        .map_err(map_vector_mutation)?,
                )?;
            } else {
                return Err(SignalError::invalid_input("epoch virtual append changed"));
            }
            self.occupied_set_count = self
                .occupied_set_count
                .checked_add(1)
                .ok_or_else(|| SignalError::internal("epoch cause occupancy overflow"))?;
        }
        Ok(())
    }

    /// Install one final consumer payload after all private slot transitions.
    pub(crate) fn finish_epoch_virtual_slot(
        &mut self,
        handle: PendingCauseSetId,
        causes: NormalizedCauseSet,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        if causes.is_empty() {
            return Err(SignalError::invalid_input(
                "empty final epoch cause payload",
            ));
        }
        let index = handle
            .index
            .ok_or_else(|| SignalError::invalid_input("missing final epoch cause slot"))?
            .get() as usize
            - 1;
        if self.slot_generations.get(index).copied() != Some(handle.generation) {
            return Err(SignalError::invalid_input("final epoch cause slot changed"));
        }
        let causes = causes.into_vec();
        self.add_epoch_references_from(&causes, work)?;
        self.remove_epoch_references_at(index, work)?;
        accounted_vector(
            self.sets
                .edit_with_retained_charge(index, work, |set| *set = causes)
                .map_err(map_vector_mutation)?,
        )?;
        Ok(())
    }

    pub(crate) fn release_epoch_accounted(
        &mut self,
        current: PendingCauseSetId,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        let Some(index) = current.index else {
            return Ok(());
        };
        self.get(current)?;
        if self.slot_generations.len() != self.sets.len() {
            return Err(SignalError::invalid_input("incomplete cause slot metadata"));
        }
        let index = index.get() as usize - 1;
        self.remove_epoch_references_at(index, work)?;
        accounted_vector(
            self.sets
                .edit_with_retained_charge(index, work, |set| {
                    *set = Vec::new();
                })
                .map_err(map_vector_mutation)?,
        )?;
        self.occupied_set_count = self.occupied_set_count.saturating_sub(1);
        accounted_vector(
            self.slot_generations
                .edit_with_retained_charge(index, work, |generation| {
                    *generation = generation.wrapping_add(1)
                })
                .map_err(map_vector_mutation)?,
        )?;
        accounted_vector(
            self.free_indices
                .push_with_retained_charge(index as u32, work)
                .map_err(map_vector_mutation)?,
        )?;
        Ok(())
    }

    pub(crate) fn publish_epoch_output_commit_accounted(
        &mut self,
        delta: ProducedAspectDelta,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        self.publish_output_commit_ordinal(delta.output_commit_ordinal);
        #[cfg(test)]
        self.published_order_probe
            .push((delta.output_commit_ordinal.0, delta.producer));
        if self
            .output_commit_reference_counts
            .contains_key(&delta.output_commit_ordinal.0)
        {
            accounted_map(
                self.published_output_commits
                    .insert_with_retained_charge(delta.output_commit_ordinal.0, delta, work)
                    .map_err(map_map_mutation)?,
            )?;
        }
        Ok(())
    }

    fn add_epoch_references_from(
        &mut self,
        causes: &[ResolvedDependencyCause],
        work: &mut Work,
    ) -> Result<(), SignalError> {
        for cause in causes {
            let ordinal = cause.binding_axes.output_commit_ordinal.0;
            let count = self
                .output_commit_reference_counts
                .get(&ordinal)
                .copied()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or_else(|| SignalError::invalid_input("cause reference overflow"))?;
            accounted_map(
                self.output_commit_reference_counts
                    .insert_with_retained_charge(ordinal, count, work)
                    .map_err(map_map_mutation)?,
            )?;
        }
        Ok(())
    }

    fn remove_epoch_references_at(
        &mut self,
        index: usize,
        work: &mut Work,
    ) -> Result<(), SignalError> {
        for position in 0..self.sets[index].len() {
            let ordinal = self.sets[index][position]
                .binding_axes
                .output_commit_ordinal
                .0;
            let count = *self
                .output_commit_reference_counts
                .get(&ordinal)
                .ok_or_else(|| SignalError::invalid_input("missing cause reference"))?;
            if count > 1 {
                accounted_map(
                    self.output_commit_reference_counts
                        .insert_with_retained_charge(ordinal, count - 1, work)
                        .map_err(map_map_mutation)?,
                )?;
            } else {
                accounted_map(
                    self.output_commit_reference_counts
                        .remove_with_retained_charge(&ordinal, work)
                        .map_err(map_map_mutation)?,
                )?;
                accounted_map(
                    self.published_output_commits
                        .remove_with_retained_charge(&ordinal, work)
                        .map_err(map_map_mutation)?,
                )?;
            }
        }
        Ok(())
    }
}
