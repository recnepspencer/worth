//! Root-custody handoffs for checkpoint, index and applied authority consumers.
mod exact_observation;

use super::root::RelationalBranchRoot;
use crate::history::data::BranchId;
use crate::identity::data::VersionId;
use crate::visibility::materialization::read_records::projection::VisibilityProjectionView;
use crate::visibility::snapshot_states::{
    ResidentVisibilitySnapshotBasis, SnapshotStateBasis, VisibilitySnapshotStateKey,
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
        match self.snapshot_basis() {
            SnapshotStateBasis::Exact(basis) => Some(basis.root()),
            SnapshotStateBasis::Historical(basis) => basis.retained_root(),
        }
    }
}

impl ResidentVisibilitySnapshotBasis {
    pub(crate) fn root(&self) -> &Arc<RelationalBranchRoot> {
        self.basis().root()
    }
}
