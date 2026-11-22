use super::{WorthQueryGraphWorkBranchAffinity, WorthQueryManagedGraphWorkSession};

impl WorthQueryManagedGraphWorkSession {
    pub const fn branch(&self) -> &WorthQueryGraphWorkBranchAffinity {
        &self.branch
    }

    pub fn admits_authorization_snapshot(
        &self,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> bool {
        self.authorization_branch.admits_snapshot(snapshot)
    }
}
