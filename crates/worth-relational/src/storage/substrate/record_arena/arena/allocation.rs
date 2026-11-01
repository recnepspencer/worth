use super::payload_allocation::diagnostic_enrichment_bytes;
use super::{RecordArena, RecordKind};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct RecordArenaAllocationInventory {
    pub(crate) authoritative_bytes: u64,
    pub(crate) diagnostic_bytes: u64,
    pub(crate) retention_metadata_bytes: u64,
    pub(crate) allocator_bookkeeping_bytes: u64,
}

impl RecordArenaAllocationInventory {
    pub(crate) fn saturating_add(self, other: Self) -> Self {
        Self {
            authoritative_bytes: self
                .authoritative_bytes
                .saturating_add(other.authoritative_bytes),
            diagnostic_bytes: self.diagnostic_bytes.saturating_add(other.diagnostic_bytes),
            retention_metadata_bytes: self
                .retention_metadata_bytes
                .saturating_add(other.retention_metadata_bytes),
            allocator_bookkeeping_bytes: self
                .allocator_bookkeeping_bytes
                .saturating_add(other.allocator_bookkeeping_bytes),
        }
    }
}

impl<K: RecordKind> RecordArena<K> {
    pub(crate) fn allocation_inventory(&self) -> RecordArenaAllocationInventory {
        struct Bytes(u64);
        impl crate::storage::substrate::StorageAllocationVisitor for Bytes {
            fn visit(
                &mut self,
                allocation: crate::storage::substrate::StorageAllocationObservation,
            ) -> bool {
                self.0 = self.0.saturating_add(allocation.bytes);
                true
            }
        }
        let mut observed = Bytes(0);
        self.visit_authoritative_allocations(false, &mut observed);
        let authoritative_bytes = observed.0;
        let diagnostic_bytes = self
            .diagnostics_enrichment
            .allocation_bytes()
            .saturating_add(
                self.diagnostics_enrichment
                    .iter()
                    .map(diagnostic_enrichment_bytes)
                    .sum(),
            );
        let retention_metadata_bytes = [
            self.branch_pins.allocation_bytes(),
            self.replay_pins.allocation_bytes(),
            self.snapshot_pins.allocation_bytes(),
        ]
        .into_iter()
        .fold(0_u64, u64::saturating_add);
        RecordArenaAllocationInventory {
            authoritative_bytes,
            diagnostic_bytes,
            retention_metadata_bytes,
            allocator_bookkeeping_bytes: 0,
        }
    }
}
