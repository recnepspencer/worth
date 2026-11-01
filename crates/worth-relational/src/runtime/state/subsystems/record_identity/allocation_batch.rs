//! The allocation operations used by the one mutation algorithm.

use crate::history::data::RecordAllocationClass;
use crate::identity::data::PartitionId;
use crate::transactions::data::{RecordAllocationDenial, RecordRef};

use super::pending_allocations::{PendingRecordAllocations, ReservedRecordAllocation};

mod sealed {
    pub trait Batch {}
    impl Batch for super::PendingRecordAllocations {}
    impl Batch for super::super::original_cursor::OriginalRecordAllocationCursor<'_> {}
}

/// A borrowed mutation input, not an allocation authority factory. The live
/// batch retains its actual reservations; the original cursor consumes only
/// recorded exact IDs and never owns or reconstructs the live allocator.
pub(crate) trait MutationRecordAllocations: sealed::Batch {
    fn reserve(
        &mut self,
        class: RecordAllocationClass,
        partition_id: PartitionId,
    ) -> Result<ReservedRecordAllocation, RecordAllocationDenial>;

    fn record(&mut self, record: RecordRef) -> Result<(), RecordAllocationDenial>;

    fn finish_mutation(&self) -> Result<(), RecordAllocationDenial>;
}

impl MutationRecordAllocations for PendingRecordAllocations {
    fn reserve(
        &mut self,
        class: RecordAllocationClass,
        partition_id: PartitionId,
    ) -> Result<ReservedRecordAllocation, RecordAllocationDenial> {
        PendingRecordAllocations::reserve(self, class, partition_id)
    }

    fn record(&mut self, record: RecordRef) -> Result<(), RecordAllocationDenial> {
        PendingRecordAllocations::record(self, record);
        Ok(())
    }

    fn finish_mutation(&self) -> Result<(), RecordAllocationDenial> {
        PendingRecordAllocations::finish_mutation(self)
    }
}
