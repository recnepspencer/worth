use super::{PhysicalReadPlanReleaseReceipt, ReadPlanCounterSnapshot};

/// Completion of local read-plan bookkeeping, not evidence of guarded byte I/O.
///
/// This value neither retains a Store root nor authorizes reclamation. Runtime
/// adapters carry their actual byte-execution evidence separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalReadPlanCompletionReceipt {
    release: PhysicalReadPlanReleaseReceipt,
    counters: ReadPlanCounterSnapshot,
}

impl PhysicalReadPlanCompletionReceipt {
    pub(super) const fn new(
        release: PhysicalReadPlanReleaseReceipt,
        counters: ReadPlanCounterSnapshot,
    ) -> Self {
        Self { release, counters }
    }

    pub const fn read_plan_release(self) -> PhysicalReadPlanReleaseReceipt {
        self.release
    }

    pub const fn counters(self) -> ReadPlanCounterSnapshot {
        self.counters
    }
}
