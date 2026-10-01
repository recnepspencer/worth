use super::PartitionInterner;
use crate::data::retained_storage::SignalConditionalRetentionReservation;

impl PartitionInterner {
    pub(crate) fn fork_reserved(
        &mut self,
        resources: &mut SignalConditionalRetentionReservation,
    ) -> Self {
        Self {
            segments: self.segments.fork_reserved(resources),
            segment_lookup: self.segment_lookup.fork_reserved(resources),
        }
    }
}
