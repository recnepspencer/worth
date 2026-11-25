use crate::history::data::BranchId;
use crate::identity::data::VersionId;
use crate::snapshots::data::SnapshotReadPolicy;

/// The exact visibility state one snapshot handle binds to.

#[derive(Debug)]
pub struct SnapshotHandleBinding {
    pub branch_id: BranchId,
    pub version_id: VersionId,
    pub read_policy: SnapshotReadPolicy,
    pub basis: crate::history::retention::VisibilitySnapshotBasis,
}

impl SnapshotHandleBinding {
    pub fn new(
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

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait SnapshotHandleBindingFixturePort: Sized {
    fn for_registry_scale_test_for_test(&self, version_id: VersionId) -> Self;
}

impl SnapshotHandleBindingFixturePort for SnapshotHandleBinding {
    fn for_registry_scale_test_for_test(&self, version_id: VersionId) -> Self {
        self.for_registry_scale_test(version_id)
    }
}
