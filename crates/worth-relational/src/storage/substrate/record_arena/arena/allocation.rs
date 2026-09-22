use super::payload_allocation::{
    aspect_version_bytes, diagnostic_enrichment_bytes, metadata_history_bytes,
};
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
        let authoritative_bytes = [
            self.slots.allocation_bytes(),
            self.partition_ids.allocation_bytes(),
            self.generations.allocation_bytes(),
            self.lifecycle.allocation_bytes(),
            self.kind_ids.allocation_bytes(),
            self.metadata_history.allocation_bytes(),
            self.created_at.allocation_bytes(),
            self.retired_at.allocation_bytes(),
            self.extra.allocation_bytes(),
            self.aspect_versions.allocation_bytes(),
            self.live_bitset.authoritative_allocation_bytes(),
            self.reclaimable_bitset.authoritative_allocation_bytes(),
            self.metadata_history
                .iter()
                .map(metadata_history_bytes::<K>)
                .sum(),
            self.extra.iter().map(K::extra_owned_allocation_bytes).sum(),
            self.aspect_versions.iter().map(aspect_version_bytes).sum(),
        ]
        .into_iter()
        .fold(0_u64, u64::saturating_add);
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
