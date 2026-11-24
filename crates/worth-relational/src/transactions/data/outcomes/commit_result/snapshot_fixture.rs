use super::CommitResult;

impl CommitResult {
    pub(crate) fn copy_with_altered_snapshot(
        &self,
        snapshot: crate::snapshots::data::SnapshotHandle,
    ) -> Self {
        let mut altered = self.clone();
        altered.outcome.snapshot = snapshot;
        altered
    }
}
