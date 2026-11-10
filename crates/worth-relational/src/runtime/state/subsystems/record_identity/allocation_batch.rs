//! The allocation operations used by the one mutation algorithm.

use crate::history::data::RecordAllocationClass;
use crate::identity::data::PartitionId;
use crate::transactions::data::{RecordAllocationDenial, RecordRef};

use super::pending_allocations::{PendingRecordAllocations, ReservedRecordAllocation};

mod sealed {
    pub trait Batch {}
    impl Batch for super::PendingRecordAllocations {}
}

/// A borrowed mutation input, not an allocation authority factory. The live
/// batch retains its actual reservations.
pub(crate) trait MutationRecordAllocations: sealed::Batch {
    fn reserve(
        &mut self,
        class: RecordAllocationClass,
        partition_id: PartitionId,
    ) -> Result<ReservedRecordAllocation, RecordAllocationDenial>;

    fn record(&mut self, record: RecordRef) -> Result<(), RecordAllocationDenial>;
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
}
