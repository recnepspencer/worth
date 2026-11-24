//! Exact projection is immutable native custody, with no owner borrow.
use super::VisibilityProjectionView;
use crate::identity::data::VersionId;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::visibility::snapshot_states::{ResidentVisibilitySnapshotBasis, SnapshotStateBasis};
pub(crate) struct PinnedProjectionSource<'a> {
    pub(crate) resident: &'a ResidentVisibilitySnapshotBasis,
    pub(crate) runtime_identity: u64,
    pub(crate) current_version: VersionId,
    pub(crate) instrumentation: &'a RuntimeInstrumentation,
}

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

    pub(in crate::visibility::materialization::read_records) fn from_pinned<'a>(
        read: impl Into<PinnedProjectionSource<'a>>,
    ) -> Self {
        let source = read.into();
        Self::from_resident_basis(
            source.resident,
            source.runtime_identity,
            source.current_version,
            source.instrumentation,
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
