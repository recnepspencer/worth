use crate::runtime::RelationalRuntime;
use crate::snapshots::data::{SnapshotHandle, SnapshotId, SnapshotReadPolicy};
use crate::storage::overlay::PartitionAccess;
use crate::storage::partition::DenseSlotBitSet;

use super::pin_assembly::{assemble_snapshot_state, SelectedRelationSlots};
use super::{SnapshotState, SnapshotStateBasis, VisibilitySnapshotBasis};

pub(crate) fn build_visibility_state(
    runtime: &RelationalRuntime,
    basis: VisibilitySnapshotBasis,
    snapshot_id: SnapshotId,
    read_policy: SnapshotReadPolicy,
) -> SnapshotState {
    let root = basis.root().clone();
    if let Some(root_version) = root
        .axes()
        .map(|axes| crate::identity::data::VersionId(axes.storage_version))
    {
        debug_assert_eq!(root_version, basis.version_id());
    }
    let handle = SnapshotHandle {
        runtime_instance_id: runtime.runtime_instance_id(),
        branch_id: basis.branch_id().clone(),
        snapshot_id,
        version_id: basis.version_id(),
        read_policy,
    };
    let entities = exact_entity_slots(root.as_ref());
    let relations = exact_relation_slots(root.as_ref());
    assemble_snapshot_state(
        handle,
        SnapshotStateBasis::Exact(basis),
        entities,
        relations,
    )
}

fn exact_entity_slots(
    state: &impl PartitionAccess,
) -> Vec<(crate::identity::data::PartitionId, DenseSlotBitSet)> {
    state
        .partition_ids()
        .into_iter()
        .filter_map(|partition_id| {
            state
                .get_partition(partition_id)
                .map(|partition| (partition_id, partition.entity_arena.live_bitset.clone()))
        })
        .collect()
}

fn exact_relation_slots(root: &crate::branch::RelationalBranchRoot) -> Vec<SelectedRelationSlots> {
    root.partition_ids()
        .into_iter()
        .filter_map(|partition_id| {
            let visible = root
                .get_partition(partition_id)?
                .relation_arena
                .live_bitset
                .clone();
            Some(SelectedRelationSlots {
                partition_id,
                visible,
                // Exact reads obtain lifecycle from this same immutable root.
                // Only historical reconstruction needs a captured override.
                retained: DenseSlotBitSet::with_capacity(0),
            })
        })
        .collect()
}
