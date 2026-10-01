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
        let bytes = scope.path().total_segment_bytes();
        let one_lookup = bytes
            .checked_mul(2)?
            .checked_add(16)?
            .checked_mul(self.paths.len().checked_add(1)?)?;
        one_lookup.checked_mul(scope.path().depth())?.checked_add(1)
    }
}
