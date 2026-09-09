//! Store work for the unique consumers in one private output packet.
use super::{CanonicalCauseSetStore, NormalizedCauseSet};
use crate::data::error::SignalError;
use crate::data::graph::storage::invalidation_causes::PendingCauseSetId;
use crate::logic::evaluation::EvaluationWork;
impl CanonicalCauseSetStore {
    pub(crate) fn admit_replacement_batch_work(
        &self,
        producer: PendingCauseSetId,
        replacements: &[(PendingCauseSetId, &NormalizedCauseSet)],
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        if replacements.is_empty() || matches!(work, EvaluationWork::Ordinary) {
            return Ok(());
        }
        work.reserve(replacements.len().checked_add(2))?;
        if self.slot_generations.len() != self.sets.len() {
            return Err(SignalError::invalid_input(
                "conditional cause replacement requires complete slot generations",
            ));
        }
        let lookup = self
            .sets
            .lookup_steps()
            .checked_add(self.slot_generations.lookup_steps());
        work.reserve(lookup)?;
        let mut touches = self.get(producer)?.len();
        for (current, next) in replacements {
            work.reserve(lookup)?;
            let old = self.get(*current)?;
            let count = old
                .len()
                .checked_add(next.len())
                .and_then(|n| touches.checked_add(n));
            work.reserve(Some(1))?;
            touches =
                count.ok_or_else(|| SignalError::internal("cause replacement count overflow"))?;
        }
        let edits = replacements.len().checked_add(1);
        let growth = touches
            .checked_add(replacements.len())
            .and_then(|n| n.checked_add(1));
        let growth_steps = growth.map(crate::data::retained_storage::ordered_lookup_steps);
        let map_steps = self
            .output_commit_reference_counts
            .lookup_steps()
            .checked_add(self.published_output_commits.lookup_steps())
            .and_then(|n| {
                growth_steps
                    .and_then(|g| g.checked_mul(6))
                    .and_then(|g| n.checked_add(g))
            });
        // Fixed ordinal keys and usize values. Includes insertion/get_mut,
        // removal, and interval retirement/readmission after earlier edits.
        work.reserve(
            map_steps
                .and_then(|n| n.checked_mul(32))
                .and_then(|n| n.checked_mul(touches)),
        )?;
        for (current, _) in replacements {
            work.reserve(lookup)?;
            let old = self.get(*current)?;
            work.reserve(Some(old.len()))?;
            for cause in old {
                work.reserve(
                    cause
                        .changed_scopes
                        .as_slice()
                        .len()
                        .checked_mul(4)
                        .and_then(|n| n.checked_add(8)),
                )?;
                if let Some(delta) = self
                    .published_output_commits
                    .get(&cause.binding_axes.output_commit_ordinal.0)
                {
                    crate::logic::invalidation::causality::admit_delta_copy(delta, work)?;
                }
            }
        }
        // Every replacement may reuse, append, or retire one slot. Producer
        // release may add a free slot first. Fixed page handles are copied;
        // cause payloads are moved by replace_discard rather than cloned.
        for (steps, len, width, exclusive) in [
            (
                self.sets.lookup_steps(),
                self.sets.len(),
                std::mem::size_of::<
                    Vec<crate::data::proof::invalidation::binding::ResolvedDependencyCause>,
                >(),
                self.sets.exclusive_capacity().is_some(),
            ),
            (
                self.slot_generations.lookup_steps(),
                self.slot_generations.len(),
                4,
                self.slot_generations.exclusive_capacity().is_some(),
            ),
            (
                self.free_indices.lookup_steps(),
                self.free_indices.len(),
                4,
                self.free_indices.exclusive_capacity().is_some(),
            ),
        ] {
            work.reserve(
                growth_steps
                    .and_then(|g| steps.checked_add(g))
                    .and_then(|n| n.checked_mul(64))
                    .and_then(|n| n.checked_add(4096))
                    .and_then(|n| edits.and_then(|e| n.checked_mul(e))),
            )?;
            // Only flat vectors can relocate their entire allocation on growth.
            if exclusive {
                work.reserve(edits.and_then(|e| {
                    len.checked_add(e)
                        .and_then(|n| n.checked_mul(width))
                        .and_then(|n| n.checked_mul(e))
                }))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
