use super::*;
use crate::identity::data::VersionId;

impl super::ProjectionRecordSource for VisibilityProjectionView {
    fn selected_root(&self) -> Option<&std::sync::Arc<crate::branch::RelationalBranchRoot>> {
        self.selected_root()
    }
    fn is_exact_basis(&self) -> bool {
        self.is_exact_basis()
    }
    fn version_id(&self) -> VersionId {
        self.version_id()
    }
}
