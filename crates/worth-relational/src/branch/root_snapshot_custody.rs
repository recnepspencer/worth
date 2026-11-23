//! Root-custody handoffs for checkpoint, index and applied authority consumers.
use super::root::RelationalBranchRoot;
use crate::history::data::BranchId;
use crate::history::retention::snapshot_projection_root::SnapshotProjectionRoot;
use crate::identity::data::VersionId;
use crate::visibility::materialization::read_records::projection::VisibilityProjectionView;
use crate::visibility::snapshot_states::{
    ResidentVisibilitySnapshotBasis, VisibilitySnapshotStateKey,
};
use std::sync::Arc;

impl VisibilitySnapshotStateKey {
    pub(crate) fn for_root(
        branch: BranchId,
        version: VersionId,
        root: &RelationalBranchRoot,
    ) -> Self {
        Self::for_selection(
            branch,
            version,
            root.id(),
            root.schema_authority().authority_digest(),
        )
    }
}

impl VisibilityProjectionView {
    pub(crate) fn selected_root(&self) -> Option<&Arc<RelationalBranchRoot>> {
        self.snapshot_basis()
            .root()
            .map(SnapshotProjectionRoot::root_custody)
    }
}

impl ResidentVisibilitySnapshotBasis {
    pub(crate) fn root(&self) -> &Arc<RelationalBranchRoot> {
        self.basis().root()
    }
}
