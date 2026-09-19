use super::PartitionVersionOverrides;
use crate::data::output::PartitionSubscription;

impl PartitionVersionOverrides {
    /// Bound comparisons and bytes examined without traversing the maps. Each
    /// tree search compares a stored key at most once; a string comparison can
    /// examine at most the query's bytes in either operand. This conservative
    /// bound deliberately does not depend on std's private B-tree fanout.
    pub(crate) fn lookup_work_bound(&self, scope: Option<&PartitionSubscription>) -> Option<usize> {
        let Some(scope) = scope else {
            return Some(1);
        };
        let partition = scope.partition.0.len().checked_mul(2)?.checked_add(1)?;
        let partitions = self.partitions.len().checked_mul(partition)?;
        let details = match scope.detail.as_ref() {
            Some(detail) => self.details.len().checked_mul(
                detail
                    .len()
                    .checked_mul(2)?
                    .checked_add(partition)?
                    .checked_add(1)?,
            )?,
            None => 0,
        };
        partitions.checked_add(details)?.checked_add(1)
    }
}
