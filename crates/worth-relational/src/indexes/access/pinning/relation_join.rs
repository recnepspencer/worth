//! Snapshot entry captures the exact join generation before verification.
use super::IndexAccess;
use crate::indexes::data::{
    BoundedIndexParityMode, BoundedRelationJoinLookupDenial, BoundedRelationJoinLookupDenialKind,
    BoundedRelationJoinLookupOutcome, BoundedRelationJoinLookupRequest,
};
use crate::visibility::snapshot_states::resolve_snapshot_handle;
fn denial(
    kind: BoundedRelationJoinLookupDenialKind,
    request: &BoundedRelationJoinLookupRequest,
) -> BoundedRelationJoinLookupDenial {
    BoundedRelationJoinLookupDenial::new(kind, request.index_id())
}
impl IndexAccess<'_> {
    pub fn execute_bounded_relation_join_lookup(
        &self,
        request: BoundedRelationJoinLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BoundedRelationJoinLookupOutcome, BoundedRelationJoinLookupDenial> {
        let runtime = self.runtime;
        runtime.performance_access().count_query_index_attempt();
        let snapshot = resolve_snapshot_handle(runtime, request.snapshot()).ok_or_else(|| {
            denial(
                BoundedRelationJoinLookupDenialKind::SnapshotUnavailable,
                &request,
            )
        })?;
        let pinned = runtime.read_truth().pin_read_capability_checked(
            &snapshot,
            &[request.index_id()],
            execution,
            |definition| {
                if matches!(
                    definition.kind,
                    crate::indexes::data::DerivedIndexKind::RelationJoin(_)
                ) {
                    Ok(())
                } else {
                    Err(denial(
                        BoundedRelationJoinLookupDenialKind::CorruptIndexEntries,
                        &request,
                    ))
                }
            },
            |stop| {
                use crate::runtime::RelationalPinnedReadAdmissionDenial as Pin;
                let kind = match stop {
                    Pin::Request(stop) => BoundedRelationJoinLookupDenialKind::Execution(
                        crate::execution::denial::kernel_stop(stop),
                    ),
                    Pin::SnapshotUnavailable => {
                        BoundedRelationJoinLookupDenialKind::SnapshotUnavailable
                    }
                    Pin::IndexNotInstalled(_) => {
                        BoundedRelationJoinLookupDenialKind::IndexNotInstalled
                    }
                };
                denial(kind, &request)
            },
        )?;
        pinned.lookup_pinned_relation_join(request, parity_mode, execution)
    }
}
