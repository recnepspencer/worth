//! Exact projection is immutable native custody, with no owner borrow.
use super::{schema_custody::ProjectionSchemaCustody, VisibilityProjectionView};
use crate::runtime::RelationalPinnedRead;
impl VisibilityProjectionView {
    pub(crate) fn from_pinned(read: &RelationalPinnedRead) -> Self {
        Self {
            runtime_identity: read.context.runtime_instance_id,
            current_version: read.current_version,
            instrumentation: read.instrumentation.clone(),
            symbols: read.symbols().clone(),
            schema: ProjectionSchemaCustody::Published(
                read.basis.root().retained_schema_authority(),
            ),
            basis: crate::visibility::snapshot_states::SnapshotStateBasis::Exact(
                read.basis.clone(),
            ),
        }
    }
}
impl RelationalPinnedRead {
    /// Project this exact snapshot. No installed owner is borrowed or consulted.
    pub fn project_snapshot(
        &self,
        handle: &crate::snapshots::data::SnapshotHandle,
        _request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Option<VisibilityProjectionView> {
        (handle == self.snapshot()).then(|| VisibilityProjectionView::from_pinned(self))
    }
}
