//! Admission for retiring one selected cause set before output publication.
use super::CanonicalCauseSetStore;
use crate::data::error::SignalError;
use crate::data::graph::storage::invalidation_causes::PendingCauseSetId;
use crate::logic::evaluation::EvaluationWork;

impl CanonicalCauseSetStore {
    pub(crate) fn admit_release_work(
        &self,
        current: PendingCauseSetId,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        if current == PendingCauseSetId::EMPTY {
            return Ok(());
        }
        if matches!(work, EvaluationWork::Ordinary) {
            return Ok(());
        }
        // Conditional release cannot enter the reconstructive metadata repair.
        work.reserve(Some(2))?;
        if self.slot_generations.len() != self.sets.len() {
            return Err(SignalError::invalid_input(
                "conditional cause release requires complete slot generations",
            ));
        }
        work.reserve(
            self.sets
                .lookup_steps()
                .checked_add(self.slot_generations.lookup_steps()),
        )?;
        let causes = self.get(current)?;
        work.reserve(Some(causes.len()))?;
        // One get_mut and at most one retirement per ordinal in each map.
        // Retirement can grow its interval overlay by at most causes.len().
        let growth = crate::data::retained_storage::ordered_lookup_steps(causes.len());
        let maps = self
            .output_commit_reference_counts
            .lookup_steps()
            .checked_add(self.published_output_commits.lookup_steps())
            .and_then(|n| growth.checked_mul(4).and_then(|g| n.checked_add(g)))
            .and_then(|n| n.checked_mul(32));
        work.reserve(maps.and_then(|n| n.checked_mul(causes.len())))?;
        for cause in causes {
            // Discarding a unique cause runs destructors for its scope entries.
            // No text is copied by the release path.
            work.reserve(
                cause
                    .changed_scopes
                    .as_slice()
                    .len()
                    .checked_mul(4)
                    .and_then(|n| n.checked_add(8)),
            )?;
            let ordinal = cause.binding_axes.output_commit_ordinal.0;
            if let Some(delta) = self.published_output_commits.get(&ordinal) {
                // Conservative bound for destroying the selected delta if its
                // final reference disappears; repeated ordinals over-admit.
                crate::logic::invalidation::causality::admit_delta_copy(delta, work)?;
            }
        }
        // Both selected vector writes detach only fixed-size page handles.
        // Payload replacement discards the old cause list without cloning it.
        work.reserve(
            self.sets
                .lookup_steps()
                .checked_add(self.slot_generations.lookup_steps())
                .and_then(|n| n.checked_mul(32))
                .and_then(|n| n.checked_add(4096)),
        )?;
        let append = match self.free_indices.exclusive_capacity() {
            Some(capacity) if capacity == self.free_indices.len() => self
                .free_indices
                .len()
                .checked_add(1)
                .and_then(|n| n.checked_mul(4)),
            Some(_) => Some(1),
            None => self
                .free_indices
                .lookup_steps()
                .checked_mul(32)
                .and_then(|n| n.checked_add(2048)),
        };
        work.reserve(append)
    }
}

#[cfg(test)]
mod tests;
