use super::CanonicalCauseSetStore;
use crate::data::retained_storage::SignalConditionalRetentionReservation;

impl CanonicalCauseSetStore {
    pub(crate) fn fork_reserved(
        &mut self,
        resources: &mut SignalConditionalRetentionReservation,
    ) -> Self {
        Self {
            generation: self.generation,
            sets: self.sets.fork_reserved(resources),
            slot_generations: self.slot_generations.fork_reserved(resources),
            free_indices: self.free_indices.fork_reserved(resources),
            next_output_commit_ordinal: self.next_output_commit_ordinal,
            published_output_commits: self.published_output_commits.fork_reserved(resources),
            occupied_set_count: self.occupied_set_count,
            output_commit_reference_counts: self
                .output_commit_reference_counts
                .fork_reserved(resources),
            retained_custody: self.retained_custody.clone(),
            deserialized_quarantine: self.deserialized_quarantine,
            #[cfg(test)]
            published_order_probe: self.published_order_probe.clone(),
            #[cfg(test)]
            last_compaction_slot_visits: self.last_compaction_slot_visits,
        }
    }
}
