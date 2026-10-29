use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::VersionId;
use worth_relational::facade::snapshots::{SnapshotHandle, SnapshotId, SnapshotReadPolicy};

fn main() {
    let _forged = SnapshotHandle {
        runtime_instance_id: 1,
        branch_id: BranchId("main".to_owned()),
        snapshot_id: SnapshotId(7),
        version_id: VersionId(3),
        read_policy: SnapshotReadPolicy::ImmutablePinned,
    };
}
