//! Assemble pins under the exact basis that owns their root lifetime.
use super::exact_state_building::{exact_entity_slots, exact_relation_slots};
use super::pin_assembly::assemble_snapshot_state;
use super::{SnapshotState, SnapshotStateBasis, VisibilitySnapshotBasis};
use crate::snapshots::data::{SnapshotHandle, SnapshotId, SnapshotReadPolicy};

impl VisibilitySnapshotBasis {
    pub(crate) fn build_visibility_state(
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

impl crate::mvcc::observation::RelationalBranchObservation {
    pub(crate) fn build_visibility_state(
        &self,
        runtime_instance_id: u64,
        snapshot_id: SnapshotId,
        read_policy: SnapshotReadPolicy,
    ) -> SnapshotState {
        VisibilitySnapshotBasis::from_observation(self).build_visibility_state(
            runtime_instance_id,
            snapshot_id,
            read_policy,
        )
    }
}
