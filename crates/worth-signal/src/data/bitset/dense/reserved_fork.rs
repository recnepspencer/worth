use super::DenseBitset;
use crate::data::retained_storage::SignalConditionalRetentionReservation;

impl DenseBitset {
    pub(crate) fn fork_reserved(
        &mut self,
        resources: &mut SignalConditionalRetentionReservation,
    ) -> Self {
        Self {
            words: self.words.fork_reserved(resources),
        }
    }
}
