//! Select the original active root while holding its registration lock.
use super::ActiveSnapshotHandleRegistry;
use crate::history::retention::SnapshotHeadSelection;
use crate::snapshots::data::SnapshotId;

use std::sync::atomic::Ordering;

impl ActiveSnapshotHandleRegistry {
    pub(in crate::internal_port::runtime::state::subsystems::visibility::snapshot_handles) fn selected_head(
        &self,
        snapshot_id: SnapshotId,
    ) -> Option<SnapshotHeadSelection> {
        self.key_lookups.fetch_add(1, Ordering::Relaxed);
        self.lock()
            .handles
            .get(&snapshot_id)
            .map(|binding| binding.basis.head_selection())
    }
}
