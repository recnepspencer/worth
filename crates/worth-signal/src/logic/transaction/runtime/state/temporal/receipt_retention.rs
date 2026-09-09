use crate::data::temporal::TemporalWakeId;

use super::TemporalRuntimeState;

impl TemporalRuntimeState {
    /// Managed conditional partitions return retirement evidence to their caller
    /// without retaining an unbounded reconstruction history in the ordinary lane.
    pub(crate) fn forget_retired_wake(&mut self, wake_id: TemporalWakeId) {
        self.retired_wakes.remove(&wake_id);
    }
}
