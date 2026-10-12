//! Snapshot input resolves and pins before entering immutable field collection.
use super::IndexAccess;
use crate::indexes::data::{
    BoundedEntityFieldLookupDenial, BoundedEntityFieldLookupDenialKind,
    BoundedEntityFieldLookupOutcome, BoundedEntityFieldLookupRequest, BoundedIndexParityMode,
};
use crate::visibility::snapshot_states::resolve_snapshot_handle;
impl IndexAccess<'_> {
    pub(in crate::indexes::access) fn lookup_snapshot(
        &self,
        request: BoundedEntityFieldLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BoundedEntityFieldLookupOutcome, BoundedEntityFieldLookupDenial> {
        crate::execution::check_request(execution).map_err(|cause| {
            BoundedEntityFieldLookupDenial::new(
                BoundedEntityFieldLookupDenialKind::Execution(cause),
                request.index_id(),
            )
        })?;
        self.runtime
            .performance_access()
            .count_query_index_attempt();
        let unavailable = || {
            BoundedEntityFieldLookupDenial::new(
                BoundedEntityFieldLookupDenialKind::SnapshotUnavailable,
                request.index_id(),
            )
        };
        let snapshot =
            resolve_snapshot_handle(self.runtime, request.snapshot()).ok_or_else(unavailable)?;
        let pinned = self
            .runtime
            .read_truth()
            .pin_read_capability(&snapshot, &[request.index_id()], execution)
            .map_err(|stop| {
                use crate::runtime::RelationalPinnedReadAdmissionDenial as Pin;
                let kind = match stop {
                    Pin::SnapshotUnavailable => {
                        BoundedEntityFieldLookupDenialKind::SnapshotUnavailable
                    }
                    Pin::IndexNotInstalled(_) => {
                        BoundedEntityFieldLookupDenialKind::IndexNotInstalled
                    }
                    Pin::Request(stop) => BoundedEntityFieldLookupDenialKind::Execution(
                        crate::execution::denial::kernel_stop(stop),
                    ),
                };
                BoundedEntityFieldLookupDenial::new(kind, request.index_id())
            })?;
        pinned.lookup_pinned_entity_field(request, parity_mode, execution)
    }
}
