//! Registry borrow releases before state/head acquisition.
use super::*;
impl RelationalBranchReferenceRegistry {
    pub(crate) fn checkpoint_component_source(
        &self,
        branch: &BranchId,
    ) -> Option<super::super::reference::CheckpointComponentCellSource> {
        let state = self.read();
        state
            .cells
            .get(branch)
            .map(RelationalBranchReferenceCell::checkpoint_component_source)
    }
}
