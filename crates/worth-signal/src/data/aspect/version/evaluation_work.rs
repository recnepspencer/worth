//! Admission for scoped evaluation map writes, before output publication.
use super::{AspectVersion, ChangedRegion, PartitionSubscription, PartitionVersionOverrides};
use crate::data::error::SignalError;
use crate::data::retained_storage::ordered_lookup_steps;
use crate::logic::evaluation::EvaluationWork;

impl PartitionVersionOverrides {
    pub(crate) fn admit_evaluation_work(
        &self,
        regions: &[ChangedRegion],
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        work.reserve(Some(regions.len()))?;
        let partitions = self.partitions.len().checked_add(regions.len());
        let details = self.details.len().checked_add(regions.len());
        work.reserve(partitions.and(details).map(|_| 0))?;
        let partition_steps = ordered_lookup_steps(partitions.expect("admitted partition count"));
        let detail_steps = ordered_lookup_steps(details.expect("admitted detail count"));
        for region in regions {
            let partition_bytes = region.partition.0.len();
            admit_insert(partition_bytes, partition_steps, work)?;
            if let Some(detail) = &region.detail {
                let bytes = partition_bytes.checked_add(detail.len());
                work.reserve(bytes.map(|_| 0))?;
                admit_insert(bytes.expect("admitted key size"), detail_steps, work)?;
            } else {
                self.admit_partition_range(region, regions.len(), detail_steps, work)?;
            }
        }
        Ok(())
    }

    fn admit_partition_range(
        &self,
        region: &ChangedRegion,
        possible_new_details: usize,
        steps: usize,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        let bytes = region.partition.0.len();
        let comparison = bytes.checked_mul(2).and_then(|n| n.checked_add(16));
        // One temporary lower key and tree seek for admission, one for apply.
        work.reserve(
            bytes
                .checked_mul(2)
                .and_then(|n| n.checked_add(comparison?.checked_mul(steps)?.checked_mul(2)?)),
        )?;
        let lower = PartitionSubscription::whole_partition(region.partition.clone());
        let mut range = self.details.range(lower..);
        loop {
            // Reserve before iterator advancement and before the prefix comparison.
            work.reserve(comparison.and_then(|n| n.checked_add(steps)))?;
            let Some((scope, _)) = range.next() else {
                break;
            };
            if scope.partition != region.partition {
                break;
            }
            work.reserve(
                comparison
                    .and_then(|n| n.checked_add(steps))
                    .and_then(|n| n.checked_add(std::mem::size_of::<AspectVersion>())),
            )?;
        }
        // Earlier regions in this same update may add details absent above.
        // Also cover apply's terminating next/prefix comparison.
        work.reserve(possible_new_details.checked_add(1).and_then(|n| {
            n.checked_mul(
                comparison?
                    .checked_add(steps)?
                    .checked_add(std::mem::size_of::<AspectVersion>())?,
            )
        }))
    }
}

fn admit_insert(
    bytes: usize,
    steps: usize,
    work: &mut EvaluationWork<'_>,
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
