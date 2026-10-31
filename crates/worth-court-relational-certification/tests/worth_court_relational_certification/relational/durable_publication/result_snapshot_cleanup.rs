//! Original-result cleanup over the externally retained actual durable medium.
use super::*;
use std::sync::Arc;
use worth_relational::facade::{
    branch::{
        RelationalCommitResultSnapshotCleanup as Task,
        RelationalCommitResultSnapshotCleanupDenial as Denial,
        RelationalCommitResultSnapshotCleanupPending as Pending,
        RelationalCommitResultSnapshotCleanupProgress as Progress,
        RelationalCommitResultSnapshotRelease as Release,
    },
    runtime::{RelationalRuntime, RelationalRuntimeCloseDenial},
    transactions::CommitResult,
};

fn finish(
    mut task: Task,
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> (Arc<CommitResult>, Release) {
    for _ in 0..8 {
        match task.advance(execution, runtime) {
            Progress::Pending {
                task: next,
                cause: Pending::Advanced,
            } => task = next,
            Progress::Complete { result, release } => return (result, release),
            other => panic!("unexpected owner progress: {other:?}"),
        }
    }
    panic!("one original binding has a finite number of retirement phases")
}

#[test]
fn result_snapshot_cleanup_preserves_same_result_across_stop_and_observer_loss() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let result = Arc::new(create_entity_outcome(
        execution,
        &runtime,
        "cleanup-original",
    ));
    let port = runtime.owner_component_services().basis_port();
    let cancellation = worth_execution::CancellationSource::new();
    let stopped_host = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        cancellation.token(),
        None,
    );
    cancellation.cancel();
    let stopped = &ExecutionRequest::serial(&stopped_host);
    let task = port.begin_commit_result_snapshot_cleanup(result.clone());
    let Progress::Pending {
        task,
        cause: Pending::Stopped(_),
    } = task.advance(stopped, &runtime)
    else {
        panic!("original stop must retain the unremoved registration")
    };
    let retained = runtime
        .snapshots()
        .retained_snapshot_for_commit(runtime.runtime_instance_id(), &result.commit)
        .unwrap();
    assert_eq!(retained.snapshot_handle(), &result.snapshot);
    assert_eq!(retained.commit(), &result.commit);
    drop(retained);
    let Progress::Pending {
        task,
        cause: Pending::Advanced,
    } = task.advance(execution, &runtime)
    else {
        panic!("actual original registration is removed first")
    };
    let Progress::Pending {
        task,
        cause: Pending::Stopped(_),
    } = task.advance(stopped, &runtime)
    else {
        panic!("stop after removal retains actual retirement")
    };
    drop(task);
    // The concrete caller retains this original Arc outside disposable task
    // observation; rejoining addresses the SAME Native-issued release cell.
    let before = medium.offered().len();
    let (returned, release) = finish(
        port.begin_commit_result_snapshot_cleanup(result.clone()),
        execution,
        &runtime,
    );
    assert!(Arc::ptr_eq(&returned, &result));
    assert!(matches!(release, Release::AlreadyReleased { .. }));
    assert_eq!(
        medium.offered().len(),
        before,
        "cleanup never republishes Native effects"
    );
    let (again, _) = finish(
        port.begin_commit_result_snapshot_cleanup(result.clone()),
        stopped,
        &runtime,
    );
    assert!(
        Arc::ptr_eq(&again, &result),
        "actual completion precedes a late stop"
    );
    runtime
        .prepare_close()
        .unwrap_or_else(|refused| panic!("unexpected close refusal: {:?}", refused.reason()))
        .finish();
}

#[test]
fn result_snapshot_cleanup_keeps_independent_reader_and_refuses_foreign_owner() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let foreign = memory_runtime();
    let result = Arc::new(create_entity_outcome(
        execution,
        &runtime,
        "independent-reader",
    ));
    let port = runtime.owner_component_services().basis_port();
    let (recovered, authority) = recover(execution, &medium);
    drop(authority);
    let task = recovered
        .owner_component_services()
        .basis_port()
        .begin_commit_result_snapshot_cleanup(result.clone());
    assert!(matches!(task.advance(execution, &recovered), Progress::Refused { cause: Denial::Release(worth_relational::facade::branch::RelationalCommitResultSnapshotReleaseDenial::Snapshot(worth_relational::facade::snapshots::RelationalSnapshotReleaseDenial::ForeignRuntime { .. })), .. }), "source readmission cannot reissue the original published cell in a new runtime");
    recovered.prepare_close().unwrap().finish();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let independent = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    drop(basis);
    let task = port.begin_commit_result_snapshot_cleanup(result.clone());
    let Progress::Refused {
        task,
        cause: Denial::OwnerBindingMismatch,
    } = task.advance(execution, &foreign)
    else {
        panic!("actual foreign owner cannot retire the result")
    };
    let competing = port.begin_commit_result_snapshot_cleanup(result.clone());
    let (_, release) = finish(task, execution, &runtime);
    assert!(matches!(release, Release::Released(_)));
    let (_, release) = finish(competing, execution, &runtime);
    assert!(matches!(release, Release::AlreadyReleased { .. }));
    let view = runtime
        .read_truth()
        .read_snapshot(&independent, execution)
        .unwrap()
        .unwrap();
    assert_eq!(view.entities().len(), 1);
    drop(view);
    let refused = runtime.prepare_close().unwrap_err();
    assert_eq!(
        refused.reason(),
        RelationalRuntimeCloseDenial::ReadCustodyRetained
    );
    let runtime = refused.into_runtime();
    runtime.snapshots().release_snapshot(&independent).unwrap();
    assert_eq!(runtime.retention().active_snapshot_count(), 0);
    assert!(runtime
        .read_truth()
        .read_snapshot(&independent, execution)
        .unwrap()
        .is_none());
    // The real materialized cache still owns its own observation. Original
    // result completion does not discharge this independent Native custody.
    let refused = runtime.prepare_close().unwrap_err();
    assert_eq!(
        refused.reason(),
        RelationalRuntimeCloseDenial::ReadCustodyRetained
    );
    // Native final destruction is deliberately unbudgeted fixture teardown;
    // this result-only task does not claim bounded cache eviction/Native finish.
    drop(refused.into_runtime());
    foreign.prepare_close().unwrap().finish();
}

#[test]
fn result_snapshot_cleanup_retains_original_root_across_head_move_delete_and_reclamation() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let source = durable_runtime(&medium);
    create_entity(execution, &source, "cold-inherited-history");
    medium.file_checkpoint(capture(execution, &source));
    drop(source);
    let (mut runtime, authority) = recover(execution, &medium);
    drop(authority);
    let result = Arc::new(create_entity_outcome(execution, &runtime, "original-root"));
    let (_, fork_source) = runtime
        .observe_fork_source(runtime.main_branch_identity().branch_id())
        .unwrap();
    let target = runtime
        .fork_branch(
            execution,
            worth_relational::facade::history::BranchId("cleanup-fork".into()),
            fork_source,
        )
        .unwrap()
        .target_identity()
        .clone();
    let port = runtime.owner_component_services().basis_port();
    let Progress::Pending {
        task,
        cause: Pending::Advanced,
    } = port
        .begin_commit_result_snapshot_cleanup(result.clone())
        .advance(execution, &runtime)
    else {
        panic!("pause after actual removal")
    };
    create_entity(execution, &runtime, "advanced-head");
    assert!(matches!(
        runtime.delete_branch(execution, &target).unwrap(),
        worth_relational::facade::branch::RelationalBranchDeletionOutcome::Deleted(_)
    ));
    let reclaimed = runtime.run_branch_root_reclamation_pass();
    assert!(
        reclaimed.roots_still_retained() > 0,
        "the original removed binding still owns its actual root"
    );
    let (_, current) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let view = runtime
        .read_truth()
        .read_observation(execution, &current.observation())
        .unwrap()
        .unwrap();
    assert_eq!(
        view.entities().len(),
        3,
        "actual cold history and resident tail remain readable"
    );
    drop(view);
    drop(current);
    let before = medium.offered().len();
    let (returned, _) = finish(task, execution, &runtime);
    assert!(Arc::ptr_eq(&returned, &result));
    assert_eq!(medium.offered().len(), before);
    runtime.run_branch_root_reclamation_pass();
    runtime.prepare_close().unwrap().finish();
}
