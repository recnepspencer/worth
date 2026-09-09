mod fork_growth;
mod normalization;
mod release_work;
mod replacement_work;
mod retained_publication;
mod slot_preparation;
pub(crate) use retained_publication::{
    PreparedRetainedCauseStorePublication, RetainedCauseStorePublicationDraft,
};
pub(crate) use slot_preparation::{CauseSlotPreparation, PreparedCauseSlot};
mod reserved_fork;
pub(crate) use normalization::NormalizedCauseSet;

mod retained_charge;

use serde::Serialize;

use super::PendingCauseSetId;
use crate::data::error::SignalError;
use crate::data::proof::invalidation::binding::ResolvedDependencyCause;
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;

#[cfg(test)]
#[path = "cause_sets/fork_granule_tests.rs"]
mod fork_granule_tests;

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct CanonicalCauseSetStore {
    pub(super) generation: u32,
    pub(super) sets: crate::data::persistent_vector::PersistentVector<Vec<ResolvedDependencyCause>>,
    #[serde(default)]
    pub(super) slot_generations: crate::data::persistent_vector::PersistentVector<u32>,
    #[serde(default)]
    pub(super) free_indices: crate::data::persistent_vector::PersistentVector<u32>,
    #[serde(default)]
    pub(super) next_output_commit_ordinal: u64,
    #[serde(default)]
    pub(super) published_output_commits:
        crate::data::persistent_ord_map::PersistentOrdMap<u64, ProducedAspectDelta>,
    #[serde(skip)]
    pub(super) occupied_set_count: usize,
    #[serde(skip)]
    pub(super) output_commit_reference_counts:
        crate::data::persistent_ord_map::PersistentOrdMap<u64, usize>,
    #[serde(skip)]
    pub(super) retained_custody: Option<
        std::sync::Arc<crate::data::retained_storage::SignalConditionalRetentionReservation>,
    >,
    #[serde(skip)]
    pub(super) deserialized_quarantine: bool,
    #[cfg(test)]
    #[serde(skip)]
    // Test observation only. It is not retained runtime meaning and therefore
    // must not participate in production fork admission or storage custody.
    pub(super) published_order_probe: Vec<(u64, crate::data::handle::NodeId)>,
    #[cfg(test)]
    #[serde(skip)]
    pub(super) last_compaction_slot_visits: usize,
}

impl CanonicalCauseSetStore {
    /// A rejected evaluation can retain its performed publication evidence.
    /// Preserve ordinal issuance without installing its causes or publications.
    pub(in crate::data::graph) fn preserve_output_issuance_from(&mut self, performed: &Self) {
        self.next_output_commit_ordinal = self
            .next_output_commit_ordinal
            .max(performed.next_output_commit_ordinal);
    }

    #[cfg(test)]
    pub(crate) const fn output_commit_ordinal_for_test(&self) -> u64 {
        self.next_output_commit_ordinal
    }

    #[cfg(test)]
    pub(crate) fn output_commit_reference_count_for_test(&self, ordinal: u64) -> usize {
        self.output_commit_reference_counts
            .get(&ordinal)
            .copied()
            .unwrap_or_default()
    }

    pub(crate) fn reserve_output_commit_ordinal(
        &self,
    ) -> crate::data::proof::invalidation::binding::OutputCommitOrdinal {
        let next = self
            .next_output_commit_ordinal
            .checked_add(1)
            .expect("output commit ordinal overflow");
        crate::data::proof::invalidation::binding::OutputCommitOrdinal(next)
    }

    fn publish_output_commit_ordinal(
        &mut self,
        ordinal: crate::data::proof::invalidation::binding::OutputCommitOrdinal,
    ) {
        debug_assert_eq!(ordinal, self.reserve_output_commit_ordinal());
        self.next_output_commit_ordinal = ordinal.0;
    }

    pub(crate) fn publish_output_commit(&mut self, delta: ProducedAspectDelta) {
        self.publish_output_commit_ordinal(delta.output_commit_ordinal);
        #[cfg(test)]
        self.published_order_probe
            .push((delta.output_commit_ordinal.0, delta.producer));
        if self
            .output_commit_reference_counts
            .contains_key(&delta.output_commit_ordinal.0)
        {
            self.published_output_commits
                .insert(delta.output_commit_ordinal.0, delta);
        }
    }

    pub(crate) fn published_output_lookup_steps(&self) -> usize {
        self.published_output_commits.lookup_steps()
    }

    pub(crate) fn published_output_commit(
        &self,
        ordinal: crate::data::proof::invalidation::binding::OutputCommitOrdinal,
    ) -> Option<&ProducedAspectDelta> {
        self.published_output_commits.get(&ordinal.0)
    }

    pub(crate) fn insert(
        &mut self,
        causes: impl IntoIterator<Item = ResolvedDependencyCause>,
    ) -> PendingCauseSetId {
        let causes = NormalizedCauseSet::prepare(
            causes.into_iter().collect(),
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )
        .expect("ordinary cause normalization");
        self.insert_normalized(causes)
    }

    fn insert_normalized(&mut self, causes: NormalizedCauseSet) -> PendingCauseSetId {
        self.replace_normalized(PendingCauseSetId::EMPTY, causes)
            .expect("ordinary cause slot insertion")
    }

    pub(crate) fn get(
        &self,
        id: PendingCauseSetId,
    ) -> Result<&[ResolvedDependencyCause], SignalError> {
        let Some(index) = id.index else {
            return Ok(&[]);
        };
        let slot = index.get() as usize - 1;
        let slot_generation = self
            .slot_generations
            .get(slot)
            .copied()
            .unwrap_or(self.generation);
        if id.generation != slot_generation {
            return Err(SignalError::invalid_input("stale pending cause-set handle"));
        }
        self.sets
            .get(slot)
            .map(Vec::as_slice)
            .ok_or_else(|| SignalError::invalid_input("unknown pending cause-set handle"))
    }

    pub(crate) fn replace_set(
        &mut self,
        current: PendingCauseSetId,
        causes: impl IntoIterator<Item = ResolvedDependencyCause>,
    ) -> Result<PendingCauseSetId, SignalError> {
        let causes = NormalizedCauseSet::prepare(
            causes.into_iter().collect(),
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )?;
        self.replace_normalized(current, causes)
    }

    pub(crate) fn replace_normalized(
        &mut self,
        current: PendingCauseSetId,
        causes: NormalizedCauseSet,
    ) -> Result<PendingCauseSetId, SignalError> {
        self.normalize_slot_metadata();
        let slot = self.prepare_cause_slots()?.replacement(
            current,
            causes.is_empty(),
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )?;
        self.publish_prepared_cause_slot(slot, causes)
    }

    pub(crate) fn release(&mut self, current: PendingCauseSetId) -> Result<(), SignalError> {
        let Some(index) = current.index else {
            return Ok(());
        };
        self.get(current)?;
        self.normalize_slot_metadata();
        let index = index.get() as usize - 1;
        self.remove_output_commit_references_at(index);
        self.sets.replace_discard(index, Vec::new());
        self.occupied_set_count = self.occupied_set_count.saturating_sub(1);
        self.slot_generations[index] = self.slot_generations[index].wrapping_add(1);
        self.free_indices.push_back(index as u32);
        Ok(())
    }

    pub(crate) fn has_occupied_sets(&self) -> bool {
        self.occupied_set_count != 0
    }

    pub(crate) fn operational_clone(&self) -> Self {
        Self {
            generation: self.generation,
            sets: self.sets.operational_clone(),
            slot_generations: self.slot_generations.operational_clone(),
            free_indices: self.free_indices.operational_clone(),
            next_output_commit_ordinal: self.next_output_commit_ordinal,
            published_output_commits: self.published_output_commits.operational_clone(),
            occupied_set_count: self.occupied_set_count,
            output_commit_reference_counts: self.output_commit_reference_counts.operational_clone(),
            retained_custody: None,
            deserialized_quarantine: self.deserialized_quarantine,
            #[cfg(test)]
            published_order_probe: self.published_order_probe.clone(),
            #[cfg(test)]
            last_compaction_slot_visits: self.last_compaction_slot_visits,
        }
    }

    pub(crate) fn fork_persistent(&mut self) -> Self {
        Self {
            generation: self.generation,
            sets: self.sets.fork_persistent(),
            slot_generations: self.slot_generations.fork_persistent(),
            free_indices: self.free_indices.fork_persistent(),
            next_output_commit_ordinal: self.next_output_commit_ordinal,
            published_output_commits: self.published_output_commits.fork_persistent(),
            occupied_set_count: self.occupied_set_count,
            output_commit_reference_counts: self.output_commit_reference_counts.fork_persistent(),
            retained_custody: self.retained_custody.clone(),
            deserialized_quarantine: self.deserialized_quarantine,
            #[cfg(test)]
            published_order_probe: self.published_order_probe.clone(),
            #[cfg(test)]
            last_compaction_slot_visits: self.last_compaction_slot_visits,
        }
    }

    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> Self {
        Self {
            generation: self.generation,
            sets: self.sets.clone(),
            slot_generations: self.slot_generations.clone(),
            free_indices: self.free_indices.clone(),
            next_output_commit_ordinal: self.next_output_commit_ordinal,
            published_output_commits: self.published_output_commits.fork_storage_identity(),
            occupied_set_count: self.occupied_set_count,
            output_commit_reference_counts: self
                .output_commit_reference_counts
                .fork_storage_identity(),
            retained_custody: self.retained_custody.clone(),
            deserialized_quarantine: self.deserialized_quarantine,
            published_order_probe: self.published_order_probe.clone(),
            last_compaction_slot_visits: self.last_compaction_slot_visits,
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        self.sets.shares_storage_with(&other.sets)
            && self
                .slot_generations
                .shares_storage_with(&other.slot_generations)
            && self.free_indices.shares_storage_with(&other.free_indices)
            && self
                .published_output_commits
                .ptr_eq(&other.published_output_commits)
            && self
                .output_commit_reference_counts
                .ptr_eq(&other.output_commit_reference_counts)
    }

    #[cfg(test)]
    pub(crate) fn replace_published_change_scopes_for_test(
        &mut self,
        ordinal: crate::data::proof::invalidation::binding::OutputCommitOrdinal,
        scopes: crate::data::proof::PartitionScopeSet,
    ) {
        let delta = self
            .published_output_commits
            .get_mut(&ordinal.0)
            .expect("test commit ordinal must exist");
        delta.changes.first_mut_for_test().changed_scopes = scopes;
    }

    #[cfg(test)]
    pub(crate) fn replace_published_internal_ordinal_for_test(
        &mut self,
        key: crate::data::proof::invalidation::binding::OutputCommitOrdinal,
        internal: crate::data::proof::invalidation::binding::OutputCommitOrdinal,
    ) {
        self.published_output_commits
            .get_mut(&key.0)
            .expect("test commit ordinal must exist")
            .output_commit_ordinal = internal;
    }
}
