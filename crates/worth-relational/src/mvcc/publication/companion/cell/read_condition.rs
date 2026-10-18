//! A restriction on an already prepared candidate, issued by its native cell.
use super::*;

pub(in crate::mvcc) struct CompanionReadCondition {
    core: Arc<CompanionBranchCellCore>,
    image: Arc<CompanionRootImage>,
}

impl<T: Send + Sync + 'static> CompanionBranchCell<T> {
    pub(in crate::mvcc) fn read_condition(
        &self,
        image: CompanionBranchImage<T>,
    ) -> CompanionReadCondition {
        CompanionReadCondition {
            core: Arc::clone(&self.core),
            image: image.image,
        }
    }
}

impl CompanionReadCondition {
    pub(in crate::mvcc) fn admits_basis(
        &self,
        runtime_instance_id: u64,
        branch: &BranchId,
        root_id: u64,
        commit_id: Option<CommitId>,
    ) -> bool {
        self.core.runtime_instance_id == runtime_instance_id
            && self.core.branch_id == *branch
            && self.image.root_id == root_id
            && self.image.commit_id == commit_id
    }

    pub(in crate::mvcc::publication::companion) fn matches_reserved(
        &self,
        core: &Arc<CompanionBranchCellCore>,
        image: &Arc<CompanionRootImage>,
    ) -> Result<(), CompanionPreflightStop> {
        if !Arc::ptr_eq(&self.core, core) {
            return Err(CompanionPreflightStop::ForeignCell);
        }
        if !Arc::ptr_eq(&self.image, image) {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        Ok(())
    }
}
