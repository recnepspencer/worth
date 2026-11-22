use super::payload_allocation::diagnostic_enrichment_bytes;
use super::{RecordArena, RecordKind};
use crate::storage::substrate::StorageAllocationOverflow;
#[cfg(test)]
use crate::storage::substrate::{
    StorageAllocationExtent, StorageAllocationObservation, StorageAllocationVisitor,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct RecordArenaAllocationInventory {
    pub(crate) authoritative_bytes: u64,
    pub(crate) diagnostic_bytes: u64,
    pub(crate) retention_metadata_bytes: u64,
    pub(crate) allocator_bookkeeping_bytes: u64,
}

impl RecordArenaAllocationInventory {
    pub(crate) fn checked_add(self, other: Self) -> Result<Self, StorageAllocationOverflow> {
        Ok(Self {
            authoritative_bytes: self
                .authoritative_bytes
                .checked_add(other.authoritative_bytes)
                .ok_or(StorageAllocationOverflow)?,
            diagnostic_bytes: self
                .diagnostic_bytes
                .checked_add(other.diagnostic_bytes)
                .ok_or(StorageAllocationOverflow)?,
            retention_metadata_bytes: self
                .retention_metadata_bytes
                .checked_add(other.retention_metadata_bytes)
                .ok_or(StorageAllocationOverflow)?,
            allocator_bookkeeping_bytes: self
                .allocator_bookkeeping_bytes
                .checked_add(other.allocator_bookkeeping_bytes)
                .ok_or(StorageAllocationOverflow)?,
        })
    }
}

#[cfg(test)]
struct CheckedBytes(Option<u64>);

#[cfg(test)]
impl StorageAllocationVisitor for CheckedBytes {
    fn visit(&mut self, allocation: StorageAllocationObservation) -> bool {
        self.0 = match allocation.extent {
            StorageAllocationExtent::Checked => {
                self.0.and_then(|total| total.checked_add(allocation.bytes))
            }
            StorageAllocationExtent::Overflow => None,
        };
        self.0.is_some()
    }
}

impl<K: RecordKind> RecordArena<K> {
    #[cfg(test)]
    pub(crate) fn allocation_inventory(
        &self,
    ) -> Result<RecordArenaAllocationInventory, StorageAllocationOverflow> {
        let mut observed = CheckedBytes(Some(0));
        self.visit_authoritative_allocations(false, &mut observed);
        let authoritative_bytes = observed.0.ok_or(StorageAllocationOverflow)?;
        let mut inventory = self.non_authoritative_allocation_inventory()?;
        inventory.authoritative_bytes = authoritative_bytes;
        Ok(inventory)
    }

    pub(crate) fn non_authoritative_allocation_inventory(
        &self,
    ) -> Result<RecordArenaAllocationInventory, StorageAllocationOverflow> {
        let diagnostic_entries = self
            .diagnostics_enrichment
            .iter()
            .try_fold(0_u64, |total, entries| {
                total.checked_add(diagnostic_enrichment_bytes(entries))
            })
            .ok_or(StorageAllocationOverflow)?;
        let diagnostic_bytes = self
            .diagnostics_enrichment
            .allocation_bytes()
            .checked_add(diagnostic_entries)
            .ok_or(StorageAllocationOverflow)?;
        let retention_metadata_bytes = [
            self.branch_pins.allocation_bytes(),
            self.replay_pins.allocation_bytes(),
            self.snapshot_pins.allocation_bytes(),
        ]
        .into_iter()
        .try_fold(0_u64, u64::checked_add)
        .ok_or(StorageAllocationOverflow)?;
        Ok(RecordArenaAllocationInventory {
            authoritative_bytes: 0,
            diagnostic_bytes,
            retention_metadata_bytes,
            allocator_bookkeeping_bytes: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{CheckedBytes, RecordArenaAllocationInventory};
    use crate::storage::substrate::{
        StorageAllocationExtent, StorageAllocationObservation, StorageAllocationOverflow,
        StorageAllocationVisitor,
    };

    #[test]
    fn arena_inventory_refuses_overflow_instead_of_saturating() {
        let max = RecordArenaAllocationInventory {
            authoritative_bytes: u64::MAX,
            ..Default::default()
        };
        let one = RecordArenaAllocationInventory {
            authoritative_bytes: 1,
            ..Default::default()
        };
        assert_eq!(max.checked_add(one), Err(StorageAllocationOverflow));

        let mut bytes = CheckedBytes(Some(u64::MAX));
        assert!(!bytes.visit(StorageAllocationObservation {
            id: 1,
            bytes: 1,
            unique: false,
            strong_count: 1,
            birth_seq: None,
            extent: StorageAllocationExtent::Checked,
        }));
        assert_eq!(bytes.0, None);

        let mut overflowed = CheckedBytes(Some(0));
        assert!(!overflowed.visit(StorageAllocationObservation {
            id: 2,
            bytes: u64::MAX,
            unique: false,
            strong_count: 1,
            birth_seq: None,
            extent: StorageAllocationExtent::Overflow,
        }));
        assert_eq!(overflowed.0, None);
    }
}
