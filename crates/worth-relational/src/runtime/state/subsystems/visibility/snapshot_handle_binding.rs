use crate::history::data::BranchId;
use crate::identity::data::VersionId;
use crate::snapshots::data::SnapshotReadPolicy;

/// The exact visibility state one snapshot handle binds to.

#[derive(Debug)]
pub(crate) struct SnapshotHandleBinding {
    pub(crate) branch_id: BranchId,
    pub(crate) version_id: VersionId,
    pub(crate) read_policy: SnapshotReadPolicy,
    pub(crate) basis: crate::history::retention::VisibilitySnapshotBasis,
}

impl SnapshotHandleBinding {
    pub(crate) fn new(
        basis: crate::history::retention::VisibilitySnapshotBasis,
        read_policy: SnapshotReadPolicy,
    ) -> Self {
        Self {
            branch_id: basis.branch_id().clone(),
            version_id: basis.version_id(),
            read_policy,
            basis,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_registry_scale_test(&self, version_id: VersionId) -> Self {
        let mut binding = self.clone();
        binding.version_id = version_id;
        binding
    }
}

impl Clone for SnapshotHandleBinding {
    fn clone(&self) -> Self {
        Self::new(self.basis.clone(), self.read_policy)
    }
}
