use super::CommitResult;

impl CommitResult {
    pub(crate) fn copy_with_altered_snapshot(
        &self,
        snapshot: worth_relational_branch::internal_port::snapshots::data::SnapshotHandle,
    ) -> Self {
        let mut altered = self.clone();
        altered.outcome.snapshot = snapshot;
        altered
    }
}

#[doc(hidden)]
pub trait CommitResultFixturePort: Sized {
    fn copy_with_altered_snapshot_for_test(
        &self,
        snapshot: worth_relational_branch::internal_port::snapshots::data::SnapshotHandle,
    ) -> Self;
}

impl CommitResultFixturePort for CommitResult {
    fn copy_with_altered_snapshot_for_test(
        &self,
        snapshot: worth_relational_branch::internal_port::snapshots::data::SnapshotHandle,
    ) -> Self {
        self.copy_with_altered_snapshot(snapshot)
    }
}
