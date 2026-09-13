use super::{WorthQueryGraphWorkBranchAffinity, WorthQueryManagedGraphWorkSession};

impl WorthQueryManagedGraphWorkSession {
    pub(in crate::domain_computation) const fn branch(&self) -> &WorthQueryGraphWorkBranchAffinity {
        &self.branch
    }

    pub(in crate::domain_computation) fn admits_authorization_snapshot(
        &self,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> bool {
        self.authorization_branch.admits_snapshot(snapshot)
    }
}
