//! Exact projection is immutable native custody, with no owner borrow.
use super::VisibilityProjectionView;
use crate::identity::data::VersionId;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::runtime::RelationalPinnedRead;
use crate::visibility::snapshot_states::{ResidentVisibilitySnapshotBasis, SnapshotStateBasis};
impl VisibilityProjectionView {
    pub(crate) fn from_resident_basis(
        resident: &ResidentVisibilitySnapshotBasis,
        runtime_identity: u64,
        current_version: VersionId,
        instrumentation: &RuntimeInstrumentation,
    ) -> Self {
        Self {
            runtime_identity,
            current_version,
            instrumentation: instrumentation.clone(),
            symbols: resident.symbols().clone(),
            empty_history_schema: None,
            resident: Some(resident.clone()),
            basis: SnapshotStateBasis::Exact(resident.basis().clone()),
        }
    }

    // The protected structural-walk lane still uses this call shape.
    pub(crate) fn from_pinned<Snapshot>(read: &RelationalPinnedRead<Snapshot>) -> Self {
        Self::from_resident_basis(
            read.resident_basis(),
            read.runtime_instance_id(),
            read.pinned_current_version(),
            read.pinned_instrumentation(),
        )
    }
}

impl VisibilityProjectionView {
    pub(in crate::visibility::materialization::read_records) fn retain_pinning_metadata(
        &self,
    ) -> (
        crate::identity::data::VersionId,
        crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
        crate::symbols::data::StringInterner,
    ) {
        (
            self.current_version,
            self.instrumentation.clone(),
            self.symbols.interner_snapshot(),
        )
    }
}
