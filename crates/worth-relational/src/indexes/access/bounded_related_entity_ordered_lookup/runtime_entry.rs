//! Installed entry pins its exact ordering generation, then delegates.
use super::*;
impl IndexAccess<'_> {
    pub fn execute_bounded_related_entity_ordered_lookup(
        &self,
        request: BoundedRelatedEntityOrderedLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BoundedRelatedEntityOrderedLookupOutcome, BoundedRelatedEntityOrderedLookupDenial>
    {
        self.runtime
            .performance_access()
            .count_query_index_attempt();
        let snapshot =
            resolve_snapshot_handle(self.runtime, request.snapshot()).ok_or_else(|| {
                denial(
                    BoundedRelatedEntityOrderedLookupDenialKind::SnapshotUnavailable,
                    &request,
                )
            })?;
        let read = self
            .runtime
            .read_truth()
            .pin_read_capability(&snapshot, &[request.index_id()], execution)
            .map_err(|stop| {
                use crate::runtime::RelationalPinnedReadAdmissionDenial as Pin;
                let kind = match stop {
                    Pin::Request(stop) => BoundedRelatedEntityOrderedLookupDenialKind::Execution(
                        crate::execution::denial::kernel_stop(stop),
                    ),
                    Pin::SnapshotUnavailable => {
                        BoundedRelatedEntityOrderedLookupDenialKind::SnapshotUnavailable
                    }
                    Pin::IndexNotInstalled(_) => {
                        BoundedRelatedEntityOrderedLookupDenialKind::IndexNotInstalled
                    }
                };
                denial(kind, &request)
            })?;
        read.lookup_pinned_related_ordering(request, parity_mode, execution)
    }
}
