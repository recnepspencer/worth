//! Update an immutable region inventory by changed owner paths in both directions.
use super::{PartitionState, RelationalPartitionAllocationInventory};
use crate::storage::substrate::{StorageAllocationObservation, StorageAllocationVisitor};

impl PartitionState {
    #[cfg(test)]
    pub(crate) fn allocation_inventory_since(
        &self,
        previous: &Self,
        prior: RelationalPartitionAllocationInventory,
    ) -> Result<RelationalPartitionAllocationInventory, super::PartitionContentDigestError> {
        self.allocation_inventory_and_introduced_since(previous, prior, None)
            .map(|(inventory, _)| inventory)
    }

    /// Update `prior` to `self`'s exact inventory in one forward + reverse pass,
    /// returning both the updated inventory and the forward introduced
    /// authoritative bytes so publication never computes the forward delta twice.
    pub(crate) fn allocation_inventory_and_introduced_since(
        &self,
        previous: &Self,
        prior: RelationalPartitionAllocationInventory,
        previous_watermark: Option<u64>,
    ) -> Result<(RelationalPartitionAllocationInventory, u64), super::PartitionContentDigestError>
    {
        let introduced = self.new_allocation_inventory_since(previous, previous_watermark)?;
        let removed = previous.new_allocation_inventory_since(self, None)?;
        let inventory = RelationalPartitionAllocationInventory {
            authoritative_bytes: update(
                prior.authoritative_bytes,
                removed.authoritative_bytes,
                introduced.authoritative_bytes,
            )?,
            diagnostic_bytes: update(
                prior.diagnostic_bytes,
                removed.diagnostic_bytes,
                introduced.diagnostic_bytes,
            )?,
            optional_cache_bytes: update(
                prior.optional_cache_bytes,
                removed.optional_cache_bytes,
                introduced.optional_cache_bytes,
            )?,
            // Both operands are immutable regions with cleared live-owner pins.
            retention_metadata_bytes: 0,
            allocator_bookkeeping_bytes: 0,
        };
        Ok((inventory, introduced.authoritative_bytes))
    }

    fn new_allocation_inventory_since(
        &self,
        previous: &Self,
        previous_watermark: Option<u64>,
    ) -> Result<RelationalPartitionAllocationInventory, super::PartitionContentDigestError> {
        let mut diagnostic = CheckedBytes::new();
        self.entity_arena
            .visit_new_diagnostic_allocations(&previous.entity_arena, &mut diagnostic);
        self.relation_arena
            .visit_new_diagnostic_allocations(&previous.relation_arena, &mut diagnostic);
        let mut cache = CheckedBytes::new();
        self.adjacency
            .visit_new_cache_allocations(&previous.adjacency, &mut cache);
        self.reverse_adjacency
            .visit_new_cache_allocations(&previous.reverse_adjacency, &mut cache);
        Ok(RelationalPartitionAllocationInventory {
            authoritative_bytes: self
                .new_authoritative_allocation_bytes_since_with_watermark(
                    previous,
                    previous_watermark,
                )?,
            diagnostic_bytes: diagnostic
                .finish()
                .ok_or(super::PartitionContentDigestError::AllocationAccountingOverflow)?,
            optional_cache_bytes: cache
                .finish()
                .ok_or(super::PartitionContentDigestError::AllocationAccountingOverflow)?,
            ..Default::default()
        })
    }
}

fn update(
    prior: u64,
    removed: u64,
    introduced: u64,
) -> Result<u64, super::PartitionContentDigestError> {
    prior
        .checked_sub(removed)
        .and_then(|bytes| bytes.checked_add(introduced))
        .ok_or(super::PartitionContentDigestError::AllocationAccountingOverflow)
}

struct CheckedBytes(Option<u64>);
impl CheckedBytes {
    const fn new() -> Self {
        Self(Some(0))
    }
    const fn finish(self) -> Option<u64> {
        self.0
    }
}
impl StorageAllocationVisitor for CheckedBytes {
    fn visit(&mut self, allocation: StorageAllocationObservation) -> bool {
        self.0 = match allocation.extent {
            crate::storage::substrate::StorageAllocationExtent::Checked => {
                self.0.and_then(|bytes| bytes.checked_add(allocation.bytes))
            }
            crate::storage::substrate::StorageAllocationExtent::Overflow => None,
        };
        self.0.is_some()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn invalid_inventory_deltas_refuse_without_saturating_a_truth_claim() {
        use super::super::PartitionContentDigestError::AllocationAccountingOverflow;
        assert_eq!(super::update(0, 1, 0), Err(AllocationAccountingOverflow));
        assert_eq!(
            super::update(u64::MAX, 0, 1),
            Err(AllocationAccountingOverflow)
        );
        assert_eq!(super::update(8, 3, 2), Ok(7));
    }
}
