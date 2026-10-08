//! One request-owned allowance for a collection of ordinary retained reads.
//!
//! This meter grants no query, scope, disclosure, or mutation authority. Every
//! item still needs its ordinary installed query plan. Read work is the actual
//! root/tree work; authorization and publication retain their separate owners.

use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

mod memory;
mod planning;
pub(in crate::domain_computation::primary_graph) use planning::PlannedBatchItem;
pub use planning::WorthQueryApplicationQueryBatchReadPlan;
#[cfg(test)]
mod tests;
mod work;
pub use memory::WorthQueryApplicationQueryBatchMemory;

/// Aggregate bounds, in addition to each item's installed query ceilings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryBatchLimits {
    maximum_items: NonZeroUsize,
    maximum_read_work: NonZeroUsize,
    maximum_retained_bytes: NonZeroUsize,
    maximum_result_bytes_per_item: NonZeroUsize,
}

impl WorthQueryApplicationQueryBatchLimits {
    pub const fn new(
        maximum_items: NonZeroUsize,
        maximum_read_work: NonZeroUsize,
        maximum_retained_bytes: NonZeroUsize,
        maximum_result_bytes_per_item: NonZeroUsize,
    ) -> Self {
        Self {
            maximum_items,
            maximum_read_work,
            maximum_retained_bytes,
            maximum_result_bytes_per_item,
        }
    }

    pub const fn maximum_items(self) -> usize {
        self.maximum_items.get()
    }
    pub const fn maximum_read_work(self) -> usize {
        self.maximum_read_work.get()
    }
    pub const fn maximum_retained_bytes(self) -> usize {
        self.maximum_retained_bytes.get()
    }
    pub const fn maximum_result_bytes_per_item(self) -> NonZeroUsize {
        self.maximum_result_bytes_per_item
    }
}

/// Admission failure before a claim or read exceeds its common allowance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryBatchResourceDenial {
    ItemLimit {
        required: usize,
        maximum: usize,
    },
    WorkLimit {
        required: usize,
        maximum: usize,
    },
    MemoryLimit {
        required: usize,
        maximum: usize,
    },
    CounterOverflow,
    WorkAccountingMismatch,
    /// The schedule or admitted slot belongs to another installed runtime.
    ForeignPlan,
    /// This installed query has no remaining registered read slot.
    UnplannedRead,
    /// The schedule has already been frozen or extended once.
    PlanningFrozen,
}

/// Genuine owner measurement, captured after all reads finish. Byte counts
/// are charged logical custody envelopes, not allocator or resident memory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryBatchWork {
    read_work: usize,
    retained_bytes: usize,
    peak_bytes: usize,
}

impl WorthQueryApplicationQueryBatchWork {
    pub const fn read_work_units(self) -> usize {
        self.read_work
    }
    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }
    pub const fn peak_bytes(self) -> usize {
        self.peak_bytes
    }
}

#[derive(Default)]
struct BatchTotals {
    work: usize,
    bytes: usize,
    peak_bytes: usize,
    memory_denial: Option<WorthQueryApplicationQueryBatchResourceDenial>,
}

/// A budget, not an access capability. Its storage claims live with the actual
/// buffers and disclosed sources, including sources retained after batch drop.
pub struct WorthQueryApplicationQueryBatchAdmission {
    policy: BatchPolicy,
    totals: Arc<Mutex<BatchTotals>>,
}

#[derive(Clone)]
enum BatchPolicy {
    Bounded(WorthQueryApplicationQueryBatchLimits),
    Installed(Arc<Mutex<planning::InstalledBatchPlan>>),
}

impl BatchPolicy {
    fn maximum_work(&self) -> usize {
        match self {
            Self::Bounded(limits) => limits.maximum_read_work(),
            Self::Installed(plan) => {
                plan.lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .work
            }
        }
    }
    fn maximum_bytes(&self) -> usize {
        match self {
            Self::Bounded(limits) => limits.maximum_retained_bytes(),
            Self::Installed(plan) => {
                plan.lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .bytes
            }
        }
    }
}

impl WorthQueryApplicationQueryBatchAdmission {
    pub fn new(limits: WorthQueryApplicationQueryBatchLimits) -> Self {
        Self {
            policy: BatchPolicy::Bounded(limits),
            totals: Arc::new(Mutex::new(BatchTotals::default())),
        }
    }
    pub fn maximum_result_bytes_per_item(&self) -> NonZeroUsize {
        match &self.policy {
            BatchPolicy::Bounded(limits) => limits.maximum_result_bytes_per_item(),
            BatchPolicy::Installed(plan) => {
                plan.lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .result
            }
        }
    }

    pub fn admit_items(
        &self,
        count: usize,
    ) -> Result<(), WorthQueryApplicationQueryBatchResourceDenial> {
        let maximum = match &self.policy {
            BatchPolicy::Bounded(limits) => limits.maximum_items(),
            BatchPolicy::Installed(plan) => {
                plan.lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .items
            }
        };
        if count > maximum {
            Err(WorthQueryApplicationQueryBatchResourceDenial::ItemLimit {
                required: count,
                maximum,
            })
        } else {
            Ok(())
        }
    }

    pub fn observe(&self) -> WorthQueryApplicationQueryBatchWork {
        let totals = self
            .totals
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        WorthQueryApplicationQueryBatchWork {
            read_work: totals.work,
            retained_bytes: totals.bytes,
            peak_bytes: totals.peak_bytes,
        }
    }

    pub(super) fn memory_denial(&self) -> Option<WorthQueryApplicationQueryBatchResourceDenial> {
        self.totals
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .memory_denial
    }
}
