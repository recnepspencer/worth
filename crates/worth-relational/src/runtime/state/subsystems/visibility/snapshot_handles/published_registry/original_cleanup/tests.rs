//! Genuine original registration contention, without a fabricated release cell.
use super::*;
use crate::facade::branch::{
    RelationalCommitResultSnapshotCleanupPending as Pending,
    RelationalCommitResultSnapshotCleanupProgress as Progress,
};
use crate::tests::support::*;

#[test]
fn result_snapshot_cleanup_registry_contention_preserves_exact_cell_and_competing_task() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let mut runtime = durable_runtime_with_test_schema();
    let result = Arc::new(create_entity_outcome(
        execution,
        &runtime,
        "registry-contention",
    ));
    let port = runtime.owner_component_services().basis_port();
    let task = port.begin_commit_result_snapshot_cleanup(result.clone());
    let closeout = runtime
        .visibility
        .published_snapshot_closeout(result.snapshot.snapshot_id())
        .unwrap();
    let capacity = runtime.visibility.published_snapshot_capacity_binding();
    assert_eq!(capacity.occupied_handles(), 1);
    let registry = runtime.visibility.handles.published.clone();
    let original_root = registry
        .binding(result.snapshot.snapshot_id())
        .map(|binding| Arc::downgrade(binding.basis.root()))
        .unwrap();
    let (_, fork_source) = runtime
        .observe_fork_source(runtime.main_branch_identity().branch_id())
        .unwrap();
    let target = runtime
        .fork_branch(
            execution,
            crate::history::data::BranchId("exact-cleanup-root".into()),
            fork_source,
        )
        .unwrap()
        .target_identity()
        .clone();
    let locked = registry.entries.lock().unwrap();
    let Progress::Pending {
        task,
        cause: Pending::Busy,
    } = task.advance(execution, &runtime)
    else {
        panic!("published contention must retain task without waiting")
    };
    assert_eq!(locked.by_id.len(), 1);
    assert!(!result.snapshot_release_observation().was_released());
    drop(locked);
    let Progress::Pending {
        task,
        cause: Pending::Advanced,
    } = task.advance(execution, &runtime)
    else {
        panic!("same task removes original registration")
    };
    assert_eq!(registry.count(), 0);
    assert_eq!(capacity.occupied_handles(), 0);
    let original = result.snapshot_release_observation();
    let locked = original.retirement.lock().unwrap();
    let competing = port.begin_commit_result_snapshot_cleanup(result.clone());
    let Progress::Pending {
        task: competing,
        cause: Pending::Busy,
    } = competing.advance(execution, &runtime)
    else {
        panic!("competing task must not replace or drop actual retirement")
    };
    assert!(matches!(*locked, PublishedSnapshotRetirement::Removed(_)));
    drop(locked);
    create_entity(execution, &runtime, "move-original-root-off-main");
    assert!(matches!(
        runtime.delete_branch(execution, &target).unwrap(),
        crate::branch::RelationalBranchDeletionOutcome::Deleted(_)
    ));
    runtime.run_branch_root_reclamation_pass();
    assert!(original_root.upgrade().is_some(), "the exact original issued root survives head movement, branch removal and real reclamation while its binding awaits retirement");
    // A real original closeout joins this SAME pending retirement, rather than
    // changing the release fact into a second registration or second charge.
    drop(closeout);
    assert!(matches!(
        task.advance(execution, &runtime),
        Progress::Complete { .. }
    ));
    assert!(matches!(
        competing.advance(execution, &runtime),
        Progress::Complete { .. }
    ));
    assert_eq!(registry.count(), 0);
    assert_eq!(capacity.occupied_handles(), 0);
    runtime.run_branch_root_reclamation_pass();
    assert!(original_root.upgrade().is_none(), "the SAME original root is reclaimable only after actual binding retirement; no hidden test read/root lease pins it");
    runtime.prepare_close().unwrap().finish();
}
