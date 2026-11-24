//! Root-custody handoffs for checkpoint, index and applied authority consumers.
pub mod exact_observation;

use super::root::RelationalBranchRoot;
use crate::history::data::BranchId;
use crate::identity::data::VersionId;
use crate::visibility::materialization::read_records::projection::VisibilityProjectionView;
use crate::visibility::snapshot_states::{
    ResidentVisibilitySnapshotBasis, SnapshotStateBasis, VisibilitySnapshotStateKey,
};
use std::sync::Arc;

impl VisibilitySnapshotStateKey {
    pub fn for_root(branch: BranchId, version: VersionId, root: &RelationalBranchRoot) -> Self {
        Self::for_selection(
            branch,
            version,
            root.id(),
            root.schema_authority().authority_digest(),
        )
    }
}

impl VisibilityProjectionView {
    pub(crate) fn selected_root(&self) -> Option<&RelationalBranchRoot> {
        match self.snapshot_basis() {
            SnapshotStateBasis::Exact(basis) => Some(basis.root()),
            SnapshotStateBasis::Historical(basis) => basis.retained_root().map(Arc::as_ref),
        }
    }

    pub(crate) fn retain_selected_root(&self) -> Option<Arc<RelationalBranchRoot>> {
        match self.snapshot_basis() {
            SnapshotStateBasis::Exact(basis) => Some(basis.retain_root()),
            SnapshotStateBasis::Historical(basis) => basis.retained_root().cloned(),
        }
    }
}

impl ResidentVisibilitySnapshotBasis {
    pub fn root(&self) -> &RelationalBranchRoot {
        self.basis().root()
    }
}

#[doc(hidden)]
pub trait VisibilityProjectionViewOwnerPort: Sized {
    fn selected_root(&self) -> Option<&RelationalBranchRoot>;
    fn retain_selected_root(&self) -> Option<Arc<RelationalBranchRoot>>;
}

impl VisibilityProjectionViewOwnerPort for VisibilityProjectionView {
    fn selected_root(&self) -> Option<&RelationalBranchRoot> {
        self.selected_root()
    }

    fn retain_selected_root(&self) -> Option<Arc<RelationalBranchRoot>> {
        self.retain_selected_root()
    }
}
