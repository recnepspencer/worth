//! The allocation operations used by the one mutation algorithm.

use super::allocation_source::AllocationSource;
use crate::history::data::record_allocation_class::RecordAllocationClass;
use crate::identity::data::PartitionId;
use crate::transactions::data::outcomes::conflict_class::record_allocation_denial::RecordAllocationDenial;
use crate::transactions::data::primitives::RecordRef;

use super::pending_allocations::{PendingRecordAllocations, ReservedRecordAllocation};

mod sealed {
    pub trait Batch {}
    impl Batch for super::PendingRecordAllocations {}
}

/// A borrowed mutation input, not an allocation authority factory. The live
/// batch retains its actual reservations.
pub trait MutationRecordAllocations: sealed::Batch {
    fn reserve(
        &mut self,
        class: RecordAllocationClass,
        partition_id: PartitionId,
    ) -> Result<ReservedRecordAllocation, RecordAllocationDenial>;

    fn record(&mut self, record: RecordRef) -> Result<(), RecordAllocationDenial>;

    /// Name the records the next applied intent allocates, in apply order.
    fn begin_intent(&mut self, sources: Vec<AllocationSource>);
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

    fn begin_intent(&mut self, sources: Vec<AllocationSource>) {
        PendingRecordAllocations::begin_intent(self, sources);
    }
}
