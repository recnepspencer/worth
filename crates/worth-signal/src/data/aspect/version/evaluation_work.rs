//! Admission for scoped evaluation map writes, before output publication.
use super::{AspectVersion, ChangedRegion, PartitionVersionOverrides, ScopePath};
use crate::data::error::SignalError;
use crate::data::retained_storage::ordered_lookup_steps;
use crate::logic::evaluation::EvaluationWork;

impl PartitionVersionOverrides {
    pub(crate) fn admit_evaluation_work(
        &self,
        regions: &[ChangedRegion],
        work: &mut EvaluationWork<'_, '_>,
    ) -> Result<(), SignalError> {
        work.reserve(Some(regions.len()))?;
        let possible_new_paths = regions.len().checked_mul(ScopePath::MAX_DEPTH);
        work.reserve(possible_new_paths.map(|_| 0))?;
        let count = self
            .paths
            .len()
            .checked_add(possible_new_paths.expect("admitted path count"));
        work.reserve(count.map(|_| 0))?;
        let steps = ordered_lookup_steps(count.expect("admitted path count"));
        for region in regions {
            for depth in 1..=region.path().depth() {
                let bytes = region.path().segments()[..depth]
                    .iter()
                    .try_fold(0usize, |sum, segment| sum.checked_add(segment.len()));
                work.reserve(bytes.map(|_| 0))?;
                let owned_key = bytes
                    .expect("admitted path size")
                    .checked_add(depth * std::mem::size_of::<String>());
                work.reserve(owned_key.map(|_| 0))?;
                admit_insert(owned_key.expect("admitted owned path size"), steps, work)?;
            }
        }
        Ok(())
    }
}

fn admit_insert(
    bytes: usize,
    steps: usize,
    work: &mut EvaluationWork<'_, '_>,
) -> Result<(), SignalError> {
    // Owned key copy, comparisons, and fixed B-tree insertion/split movement.
    work.reserve(
        bytes
            .checked_mul(2)
            .and_then(|n| n.checked_add(16))
            .and_then(|n| n.checked_mul(steps))
            .and_then(|n| n.checked_add(steps.checked_mul(16)?))
            .and_then(|n| n.checked_add(bytes))
            .and_then(|n| n.checked_add(std::mem::size_of::<AspectVersion>())),
    )
}

#[cfg(test)]
mod tests;
