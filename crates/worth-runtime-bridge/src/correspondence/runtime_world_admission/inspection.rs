use std::sync::atomic::{AtomicU64, Ordering};

/// Read-only evidence for the Runtime World correspondence hot path.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct RuntimeWorldCorrespondenceInspectionCounters {
    binding_index_lookups: u64,
}

impl RuntimeWorldCorrespondenceInspectionCounters {
    pub const fn binding_index_lookups(self) -> u64 {
        self.binding_index_lookups
    }
}

#[derive(Debug, Default)]
pub(crate) struct RuntimeWorldCorrespondenceInspectionLedger {
    binding_index_lookups: AtomicU64,
}

impl RuntimeWorldCorrespondenceInspectionLedger {
    /// Internal recorder used only by the direct currentness index lookup.
    pub(crate) fn record_binding_index_lookup(&self) {
        self.binding_index_lookups.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn snapshot(&self) -> RuntimeWorldCorrespondenceInspectionCounters {
        RuntimeWorldCorrespondenceInspectionCounters {
            binding_index_lookups: self.binding_index_lookups.load(Ordering::Relaxed),
        }
    }
}
