//! Assemble pins under the exact basis that owns their root lifetime. Pins
//! derive from Full partitions, so callers demand the basis' regions first.
use super::exact_state_building::{exact_entity_slots, exact_relation_slots};
use super::pin_assembly::assemble_snapshot_state;
use super::{SnapshotState, SnapshotStateBasis};
use crate::history::retention::VisibilitySnapshotBasis;
use crate::snapshots::data::{SnapshotHandle, SnapshotId, SnapshotReadPolicy};

impl VisibilitySnapshotBasis {
    pub fn build_visibility_state(
        self,
        runtime_instance_id: u64,
        snapshot_id: SnapshotId,
        read_policy: SnapshotReadPolicy,
    ) -> SnapshotState {
        let root = self.projection_root();
        if let Some(root_version) = root.storage_version() {
            debug_assert_eq!(
                /* Proof: The exact basis key and root version are derived together from the same admitted immutable branch observation; callers cannot independently replace the private basis root. */
                root_version,
                self.version_id()
            );
        }
        let handle = SnapshotHandle {
            runtime_instance_id,
            branch_id: self.branch_id().clone(),
            snapshot_id,
            version_id: self.version_id(),
            read_policy,
        };
        let entities = exact_entity_slots(&root);
        let relations = exact_relation_slots(&root);
        assemble_snapshot_state(handle, SnapshotStateBasis::Exact(self), entities, relations)
    }
}
