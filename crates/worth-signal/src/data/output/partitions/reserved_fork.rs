use super::PartitionInterner;
use crate::data::retained_storage::SignalConditionalRetentionReservation;

impl PartitionInterner {
    pub(crate) fn fork_reserved(
        &mut self,
        resources: &mut SignalConditionalRetentionReservation,
    ) -> Self {
        Self {
            partitions: self.partitions.fork_reserved(resources),
            details: self.details.fork_reserved(resources),
            partition_lookup: self.partition_lookup.fork_reserved(resources),
            detail_lookup: self.detail_lookup.fork_reserved(resources),
        }
    }
}
