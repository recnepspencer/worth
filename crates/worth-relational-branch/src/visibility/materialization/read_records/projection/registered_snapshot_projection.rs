//! Select one registered exact root before creating its read projection.
use super::creation::ProjectionInputs;
use super::VisibilityProjectionView;
use crate::visibility::snapshot_states::SnapshotStateBasis;

impl VisibilityProjectionView {
    pub(crate) fn from_snapshot(
        projection: ProjectionInputs<'_>,
        directory: crate::visibility::snapshot_states::resolution::SnapshotReadDirectory<'_>,
        handle: &crate::snapshots::data::SnapshotHandle,
    ) -> Option<Self> {
        let basis = crate::visibility::snapshot_states::resolution::resolve_snapshot_basis(
            directory, handle,
        )?;
        if basis.projection_root().has_materialization_unavailable() {
            return None;
        }
        Some(Self::new(projection, SnapshotStateBasis::Exact(basis)))
    }
}
