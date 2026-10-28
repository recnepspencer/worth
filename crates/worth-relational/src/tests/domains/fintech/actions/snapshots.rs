use crate::facade::snapshots::SnapshotHandle;

use super::super::fixture::FintechWorld;

pub(crate) fn capture_world_snapshot(world: &mut FintechWorld) -> SnapshotHandle {
    world
        .runtime
        .visibility_authority()
        .snapshot()
        .expect("current snapshot fixture is admitted")
}

pub(crate) fn release_snapshot_handle(world: &mut FintechWorld, snapshot: &SnapshotHandle) -> bool {
    world
        .runtime
        .visibility_authority()
        .release_snapshot(snapshot)
        .is_ok()
}
