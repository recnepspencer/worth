//! One request-owned allowance for a collection of ordinary retained reads.
//!
//! This meter grants no query, scope, disclosure, or mutation authority. Every
//! item still needs its ordinary installed query plan. Read work is the actual
//! root/tree work; authorization and publication retain their separate owners.

use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

mod memory;
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
    ItemLimit { required: usize, maximum: usize },
    WorkLimit { required: usize, maximum: usize },
    MemoryLimit { required: usize, maximum: usize },
    CounterOverflow,
    WorkAccountingMismatch,
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
    limits: WorthQueryApplicationQueryBatchLimits,
    totals: Arc<Mutex<BatchTotals>>,
}

impl WorthQueryApplicationQueryBatchAdmission {
    pub fn new(limits: WorthQueryApplicationQueryBatchLimits) -> Self {
        Self {
            limits,
            totals: Arc::new(Mutex::new(BatchTotals::default())),
        }
    }
    pub const fn maximum_result_bytes_per_item(&self) -> NonZeroUsize {
        self.limits.maximum_result_bytes_per_item()
    }

    pub fn admit_items(
        &self,
        count: usize,
    ) -> Result<(), WorthQueryApplicationQueryBatchResourceDenial> {
        if count > self.limits.maximum_items() {
            Err(WorthQueryApplicationQueryBatchResourceDenial::ItemLimit {
                required: count,
                maximum: self.limits.maximum_items(),
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
