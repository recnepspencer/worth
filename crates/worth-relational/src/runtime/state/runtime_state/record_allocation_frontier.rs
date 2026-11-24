//! Read the partition frontier of the live runtime's record allocation owner.
use super::RelationalRuntime;
use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use crate::identity::data::PartitionId;
use worth_execution::ExecutionRequest;

impl RelationalRuntime {
    /// Observes the highest partition in the global record allocator.
    /// O(allocation-partition rows), O(1) scratch, no record-body or source reads.
    /// Includes pending/burned reservations and restored frontiers across branches.
    /// This data grants no allocation, publication or readmission authority.
    pub fn observe_record_partition_high_water(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<PartitionId>, Cause> {
        self.record_identity.observe_partition_high_water(request)
    }
}
