use crate::data::temporal::{TemporalRetiredWakeCompactionReport, TemporalWakeId};

use super::TemporalRuntimeState;

impl TemporalRuntimeState {
    /// Managed conditional partitions return retirement evidence to their caller
    /// without retaining an unbounded reconstruction history in the ordinary lane.
    pub(crate) fn forget_retired_wake(&mut self, wake_id: TemporalWakeId) {
        self.retired_wakes.remove(&wake_id);
    }

    pub(crate) fn compact_retired_wake_tail(
        &mut self,
        retained_limit: u32,
        max_expired: u32,
    ) -> TemporalRetiredWakeCompactionReport {
        let mut expired_now = 0_u32;
        while self.retired_wakes.len() > retained_limit as usize && expired_now < max_expired {
            let Some(id) = self.retired_wakes.keys().next().copied() else {
                break;
            };
            if let Some(receipt) = self.retired_wakes.remove(&id) {
                self.expired_retired_wakes
                    .absorb(b"temporal-retired-wake", id.get(), &receipt);
                expired_now += 1;
            }
        }
        TemporalRetiredWakeCompactionReport::new(
            expired_now,
            self.retired_wakes.len().min(u32::MAX as usize) as u32,
            self.expired_retired_wakes.count(),
            self.expired_retired_wakes.digest(),
        )
    }
}
