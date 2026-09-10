use std::num::NonZeroU32;

use super::{CanonicalCauseSetStore, PendingCauseSetId};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::ResolvedDependencyCause;

pub(super) struct CauseSetHandleRemap {
    pub(super) consumer: NodeId,
    pub(super) previous: PendingCauseSetId,
    pub(super) current: PendingCauseSetId,
}

impl SignalGraph {
    /// Explicit reconstruction may remap every occupied consumer. Ordinary
    /// publication retains reusable free slots and must never call this lane.
    pub(crate) fn compact_cause_set_storage(&mut self) -> Result<(), SignalError> {
        let remaps = self.cause_sets.rebuild_occupied_generation()?;
        for remap in remaps {
            if self.node_pending_cause_set_id(remap.consumer)? != remap.previous {
                return Err(SignalError::invalid_input(
                    "canonical cause-set handle does not match its consumer",
                ));
            }
            self.set_node_pending_cause_set_id(remap.consumer, remap.current)?;
        }
        Ok(())
    }
}

impl CanonicalCauseSetStore {
    pub(super) fn rebuild_occupied_generation(
        &mut self,
    ) -> Result<Vec<CauseSetHandleRemap>, SignalError> {
        for set in self.sets.iter().filter(|set| !set.is_empty()) {
            let consumer = set[0].key.consumer;
            if set.iter().any(|cause| cause.key.consumer != consumer) {
                return Err(SignalError::invalid_input(
                    "canonical cause set contains multiple consumers",
                ));
            }
        }

        let occupied_set_count = self.occupied_set_count;
        let previous_sets = std::mem::take(&mut self.sets);
        let previous_generations = std::mem::take(&mut self.slot_generations);
        #[cfg(test)]
        {
            self.last_compaction_slot_visits = previous_sets.len();
        }
        self.generation = self.generation.wrapping_add(1);
        self.free_indices.clear();
        self.occupied_set_count = 0;
        self.output_commit_reference_counts.clear();
        let mut remaps = Vec::with_capacity(occupied_set_count);
        for (index, set) in previous_sets.iter().cloned().enumerate() {
            if set.is_empty() {
                continue;
            }
            let consumer = set[0].key.consumer;
            let previous = PendingCauseSetId {
                index: NonZeroU32::new(index as u32 + 1),
                generation: previous_generations
                    .get(index)
                    .copied()
                    .unwrap_or(self.generation.wrapping_sub(1)),
            };
            let current = self.insert(set);
            remaps.push(CauseSetHandleRemap {
                consumer,
                previous,
                current,
            });
        }
        self.prune_unreferenced_output_commits();
        Ok(remaps)
    }

    pub(super) fn normalize_slot_metadata(&mut self) {
        if self.slot_generations.len() < self.sets.len() {
            let missing = self.sets.len() - self.slot_generations.len();
            self.slot_generations
                .extend(std::iter::repeat_n(self.generation, missing));
        }
    }

    pub(super) fn prune_unreferenced_output_commits(&mut self) {
        let stale = self
            .published_output_commits
            .keys()
            .filter(|ordinal| !self.output_commit_reference_counts.contains_key(ordinal))
            .copied()
            .collect::<Vec<_>>();
        for ordinal in stale {
            self.published_output_commits.remove_discard(&ordinal);
        }
    }

    pub(super) fn rebuild_derived_metadata(&mut self) {
        self.occupied_set_count = self.sets.iter().filter(|set| !set.is_empty()).count();
        self.output_commit_reference_counts.clear();
        for set in &self.sets {
            for cause in set {
                *self
                    .output_commit_reference_counts
                    .entry(cause.binding_axes.output_commit_ordinal.0)
                    .or_default() += 1;
            }
        }
        self.prune_unreferenced_output_commits();
    }

    pub(super) fn add_output_commit_references_from(&mut self, causes: &[ResolvedDependencyCause]) {
        for cause in causes {
            *self
                .output_commit_reference_counts
                .entry(cause.binding_axes.output_commit_ordinal.0)
                .or_default() += 1;
        }
    }

    pub(super) fn remove_output_commit_references_at(&mut self, index: usize) {
        for cause in &self.sets[index] {
            let ordinal = cause.binding_axes.output_commit_ordinal.0;
            let count = self
                .output_commit_reference_counts
                .get_mut(&ordinal)
                .expect("stored cause commit ordinal must be reference-counted");
            *count -= 1;
            if *count == 0 {
                self.output_commit_reference_counts.remove_discard(&ordinal);
                self.published_output_commits.remove_discard(&ordinal);
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn allocated_slot_count(&self) -> usize {
        self.sets.len()
    }

    #[cfg(test)]
    pub(crate) fn occupied_slot_count(&self) -> usize {
        self.occupied_set_count
    }

    #[cfg(test)]
    pub(crate) fn last_compaction_slot_visits(&self) -> usize {
        self.last_compaction_slot_visits
    }
}
