//! Borrow only the selected partition while the original snapshot root lives.
use crate::branch::projection_view::snapshot_head::SelectedSnapshotHead;
use crate::identity::data::PartitionId;
use crate::snapshots::data::SnapshotId;

impl super::SnapshotHandles {
    pub(crate) fn with_selected_head<R>(
        &self,
        snapshot_id: SnapshotId,
        partition: PartitionId,
        read: impl FnOnce(SelectedSnapshotHead<'_>) -> R,
    ) -> Option<R> {
        let selection = self
            .active
            .selected_head(snapshot_id)
            .or_else(|| self.published.selected_head(snapshot_id))?;
        Some(read(selection.region(partition)))
    }
}
