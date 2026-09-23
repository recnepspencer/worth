#[cfg(any(test, feature = "certification-authority"))]
use crate::ExecutedIsolationEvidenceDenial;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIsolationCounterSnapshot {
    outcome_count: u64,
    wait_count: u64,
    retry_count: u64,
    latch_counter_rows: u64,
    latch_wait_count: u64,
    reclaim_counter_rows: u64,
    blocked_maintenance_count: u64,
    reclaim_block_count: u64,
    protected_byte_footprint: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutedIsolationCounterKind {
    Outcome,
    Wait,
    Retry,
    LatchCounterRow,
    LatchWait,
    ReclaimCounterRow,
    BlockedMaintenance,
    ReclaimBlock,
    ProtectedByteFootprint,
}

impl PhysicalIsolationCounterSnapshot {
    #[cfg(any(test, feature = "certification-authority"))]
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn from_store_executed_counts(
        outcome_count: u64,
        wait_count: u64,
        retry_count: u64,
        latch_counter_rows: u64,
        latch_wait_count: u64,
        reclaim_counter_rows: u64,
        blocked_maintenance_count: u64,
        reclaim_block_count: u64,
        protected_byte_footprint: u64,
    ) -> Result<Self, ExecutedIsolationEvidenceDenial> {
        if latch_counter_rows == 0 {
            return Err(ExecutedIsolationEvidenceDenial::MissingLatchCounters);
        }
        if reclaim_counter_rows == 0 {
            return Err(ExecutedIsolationEvidenceDenial::MissingReclaimCounters);
        }
        if protected_byte_footprint == 0 {
            return Err(ExecutedIsolationEvidenceDenial::MissingProtectedByteFootprint);
        }
        Ok(Self {
            outcome_count,
            wait_count,
            retry_count,
            latch_counter_rows,
            latch_wait_count,
            reclaim_counter_rows,
            blocked_maintenance_count,
            reclaim_block_count,
            protected_byte_footprint,
        })
    }

    pub const fn outcome_count(self) -> u64 {
        self.outcome_count
    }

    pub const fn wait_count(self) -> u64 {
        self.wait_count
    }

    pub const fn retry_count(self) -> u64 {
        self.retry_count
    }

    pub const fn latch_counter_rows(self) -> u64 {
        self.latch_counter_rows
    }

    pub const fn latch_wait_count(self) -> u64 {
        self.latch_wait_count
    }

    pub const fn reclaim_counter_rows(self) -> u64 {
        self.reclaim_counter_rows
    }

    pub const fn blocked_maintenance_count(self) -> u64 {
        self.blocked_maintenance_count
    }

    pub const fn reclaim_block_count(self) -> u64 {
        self.reclaim_block_count
    }

    pub const fn protected_byte_footprint(self) -> u64 {
        self.protected_byte_footprint
    }
}
