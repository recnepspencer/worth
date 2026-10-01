use super::{PartitionInterner, PartitionSubscription};

impl PartitionInterner {
    /// Query bytes bound both operands inspected by string comparison. A base
    /// key used for retirement lookup equals the query and has the same length.
    pub(crate) fn subscription_lookup_work(
        &self,
        subscription: &PartitionSubscription,
    ) -> Option<usize> {
        subscription
            .path()
            .segments()
            .iter()
            .try_fold(0usize, |total, segment| {
                let lookup = segment
                    .len()
                    .checked_mul(2)?
                    .checked_add(1)?
                    .checked_mul(self.segment_lookup.lookup_steps())?;
                total.checked_add(lookup)
            })
    }
}
