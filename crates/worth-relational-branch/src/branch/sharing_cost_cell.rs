use crate::runtime::state::subsystems::history::history_costs::RelationalBranchSharingCostCounters;
use std::sync::{Arc, Mutex};

use crate::storage::substrate::StorageAllocationOverflow;

#[derive(Debug, Clone, Copy, Default)]
struct StoredSharingCosts {
    counters: RelationalBranchSharingCostCounters,
    authoritative_byte_overflow: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RelationalBranchSharingCostCell {
    state: Arc<Mutex<StoredSharingCosts>>,
}

impl RelationalBranchSharingCostCell {
    pub fn detached_owner_snapshot(&self) -> Self {
        let state = *self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    pub fn record(&self, update: impl FnOnce(&mut RelationalBranchSharingCostCounters)) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        update(&mut state.counters);
    }

    /// Record a cost update whose authoritative byte totals must not saturate.
    /// Returning `None` marks this cell's recorded authoritative byte totals as
    /// overflowed without mutating the byte totals to a misleading `u64::MAX`.
    pub fn record_checked_bytes(
        &self,
        update: impl FnOnce(&mut RelationalBranchSharingCostCounters) -> Option<()>,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if update(&mut state.counters).is_none() {
            state.authoritative_byte_overflow = true;
        }
    }

    pub fn snapshot(&self) -> RelationalBranchSharingCostCounters {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .counters
    }

    pub fn try_snapshot(
        &self,
    ) -> Result<RelationalBranchSharingCostCounters, StorageAllocationOverflow> {
        let state = *self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.authoritative_byte_overflow {
            Err(StorageAllocationOverflow)
        } else {
            Ok(state.counters)
        }
    }
}
