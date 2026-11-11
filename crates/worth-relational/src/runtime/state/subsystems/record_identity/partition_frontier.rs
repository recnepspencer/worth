//! Read the existing global allocation namespace, including non-main and burned rows.
use super::{PartitionId, RecordIdentitySubsystem};
use crate::execution::RelationalExecutionDenialCause as Cause;
use worth_execution::ExecutionRequest;

impl crate::runtime::RelationalRuntime {
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
impl RecordIdentitySubsystem {
    fn observe_partition_high_water(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<PartitionId>, Cause> {
        crate::execution::check_request(request)?;
        crate::execution::request_stage::optional_stage(
            request,
            |context, _memory| {
                let state = self.lock();
                let mut highest = None;
                let mut rows = state.next_slots.try_iter(|work, _bytes| {
                    crate::execution::request_stage::checkpoint(context, work).map_err(Cause::from)
                })?;
                while let Some(((_class, partition), _)) = rows.try_next(|work, _bytes| {
                    crate::execution::request_stage::checkpoint(context, work).map_err(Cause::from)
                })? {
                    highest = Some(
                        highest.map_or(*partition, |prior: PartitionId| prior.max(*partition)),
                    );
                }
                Ok::<_, Cause>(highest)
            },
            |_| 0,
        )?
    }
}
