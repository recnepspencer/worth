use super::{PartitionInterner, PartitionSubscription};

impl PartitionInterner {
    /// Query bytes bound both operands inspected by string comparison. A base
    /// key used for retirement lookup equals the query and has the same length.
    pub(crate) fn subscription_lookup_work(
        &self,
        subscription: &PartitionSubscription,
    ) -> Option<usize> {
        let partition = subscription
            .partition
            .0
            .len()
            .checked_mul(2)?
            .checked_add(1)?
            .checked_mul(self.partition_lookup.lookup_steps())?;
        let detail = match &subscription.detail {
            None => 0,
            Some(detail) => detail
                .len()
                .checked_mul(2)?
                .checked_add(1)?
                .checked_mul(self.detail_lookup.lookup_steps())?,
        };
        partition.checked_add(detail)
    }
}
