use std::sync::atomic::{AtomicUsize, Ordering};

use crate::indexes::data::DerivedIndexSelectionCounters;

#[derive(Debug, Default)]
pub(super) struct SelectionWork {
    payload_reads: AtomicUsize,
    inventory_entries: AtomicUsize,
}

impl Clone for SelectionWork {
    fn clone(&self) -> Self {
        // A runtime fork starts its own accounting lifetime.
        Self::default()
    }
}

impl SelectionWork {
    pub(super) fn read_payload(&self) {
        self.payload_reads.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn enumerate_inventory(&self, entries: usize) {
        self.inventory_entries.fetch_add(entries, Ordering::Relaxed);
    }

    pub(super) fn snapshot(&self) -> DerivedIndexSelectionCounters {
        DerivedIndexSelectionCounters {
            generation_payload_reads: self.payload_reads.load(Ordering::Relaxed),
            history_inventory_entries: self.inventory_entries.load(Ordering::Relaxed),
        }
    }
}
