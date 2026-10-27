//! Exact projection is immutable native custody, with no owner borrow.
use super::{schema_custody::ProjectionSchemaCustody, VisibilityProjectionView};
use crate::runtime::RelationalPinnedRead;
impl VisibilityProjectionView {
    pub(crate) fn from_pinned<Snapshot>(read: &RelationalPinnedRead<Snapshot>) -> Self {
        let symbols = crate::runtime::RuntimeSymbolTable::default();
        symbols.replace(read.symbols().clone());
        Self {
            runtime_identity: read.runtime_instance_id(),
            current_version: read.pinned_current_version(),
            instrumentation: read.pinned_instrumentation().clone(),
            symbols,
            schema: ProjectionSchemaCustody::Published(
                read.pinned_basis().root().retained_schema_authority(),
            ),
            resident: Some(read.resident_basis().clone()),
            basis: crate::visibility::snapshot_states::SnapshotStateBasis::Exact(
                read.pinned_basis().clone(),
            ),
        }
    }
}
impl RelationalPinnedRead {
    /// Project this exact snapshot. No installed owner is borrowed or consulted.
    pub fn project_snapshot(
        &self,
        handle: &crate::snapshots::data::SnapshotHandle,
    ) -> Option<VisibilityProjectionView> {
        (handle == self.snapshot()).then(|| VisibilityProjectionView::from_pinned(self))
    }
}

impl VisibilityProjectionView {
    pub(in crate::visibility::materialization::read_records) fn retain_pinning_metadata(
        &self,
    ) -> (
        crate::identity::data::VersionId,
        crate::runtime::RuntimeInstrumentation,
        crate::symbols::data::StringInterner,
    ) {
        (
            self.current_version,
            self.instrumentation.clone(),
            self.symbols.interner_snapshot(),
        )
    }
}
