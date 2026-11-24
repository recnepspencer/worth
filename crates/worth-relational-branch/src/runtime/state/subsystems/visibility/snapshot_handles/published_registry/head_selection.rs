//! Select the original published root while holding its registration lock.
use super::PublishedSnapshotHandleRegistry;
use crate::history::retention::SnapshotHeadSelection;
use crate::snapshots::data::SnapshotId;

impl PublishedSnapshotHandleRegistry {
    pub(in crate::internal_port::runtime::state::subsystems::visibility::snapshot_handles) fn selected_head(
        &self,
        snapshot_id: SnapshotId,
    ) -> Option<SnapshotHeadSelection> {
        self.lock()
            .by_id
            .get(&snapshot_id)
            .map(|binding| binding.binding.basis.head_selection())
    }
}
