//! Runtime cleanup checks authentic tenure and exact private snapshot registrations.
use worth_relational_publication::internal_port::transactions::data::outcomes::commit_result::CommitResultOwnerPort as _;
use worth_relational_publication::internal_port::transactions::data::outcomes::commit_result::snapshot_fixture::CommitResultFixturePort as _;
use worth_relational_branch::internal_port::runtime::state::subsystems::visibility::snapshot_handles::SnapshotHandlesFixturePort as _;
use worth_relational_branch::internal_port::runtime::state::subsystems::visibility::snapshot_handles::published_release::PublishedSnapshotReleaseObservationFixturePort as _;
use crate::branch::result_snapshot_outcome::RelationalCommitResultSnapshotCleanupPending as Pending;
use crate::branch::result_snapshot_outcome::{
    RelationalCommitResultSnapshotRelease as Outcome,
    RelationalCommitResultSnapshotReleaseDenial as Denial,
};
use crate::facade::branch::RelationalCommitResultSnapshotCleanupProgress as Progress;
use crate::tests::support::*;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

#[test]
fn result_snapshot_cleanup_refuses_an_actual_admitted_runtime_copy() {
    use crate::branch::result_snapshot_outcome::RelationalCommitResultSnapshotCleanupDenial as Denial;
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let result = std::sync::Arc::new(create_entity_outcome(
        execution,
        &runtime,
        "actual-admitted-copy",
    ));
    let operation = runtime.owner_binding().admit().unwrap();
    let admitted = crate::runtime::RelationalRuntime::admitted(
        runtime.state_binding().upgrade().unwrap(),
        operation,
    );
    let task = runtime
        .owner_component_services()
        .basis_port()
        .begin_commit_result_snapshot_cleanup(result.clone());
    let Progress::Refused {
        task,
        cause: Denial::NotOwner,
    } = task.advance(execution, &admitted)
    else {
        panic!("actual read-copy tenure is not the runtime owner")
    };
    assert!(!result.snapshot_release_observation().was_released());
    drop(admitted);
    let mut task = task;
    let mut completed = false;
    for _ in 0..8 {
        match task.advance(execution, &runtime) {
            Progress::Pending { task: next, .. } => task = next,
            Progress::Complete {
                result: returned, ..
            } => {
                assert!(std::sync::Arc::ptr_eq(&returned, &result));
                completed = true;
                break;
            }
            other => panic!("{other:?}"),
        }
    }
    assert!(completed);
    runtime.prepare_close().unwrap().finish();
}

#[test]
fn native_custody_correction_result_release_cell_tracks_public_release_closeout_and_transfer() {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let service = runtime.owner_component_services().basis_port();
    let normal = create_entity_outcome(execution, &runtime, "normal-explicit-release");
    runtime
        .snapshots()
        .release_snapshot(&normal.snapshot)
        .unwrap();
    assert!(matches!(
        service.release_commit_result_snapshot(&normal).unwrap(),
        Outcome::AlreadyReleased { .. }
    ));
    let closed = create_entity_outcome(execution, &runtime, "actual-owner-closeout");
    let closeout = runtime
        .visibility
        .published_snapshot_closeout(closed.snapshot.snapshot_id())
        .unwrap();
    drop(closeout);
    assert!(matches!(
        service.release_commit_result_snapshot(&closed).unwrap(),
        Outcome::AlreadyReleased { .. }
    ));
    let transferred = create_entity_outcome(execution, &runtime, "still-active-transfer");
    runtime
        .visibility
        .published_snapshot_closeout(transferred.snapshot.snapshot_id())
        .unwrap()
        .transfer_release_obligation();
    assert!(
        matches!(
            service
                .release_commit_result_snapshot(&transferred)
                .unwrap(),
            Outcome::Released(_)
        ),
        "moving a real release obligation is not actual release"
    );
    runtime.prepare_close().unwrap().finish();
}

#[test]
fn native_custody_correction_result_cleanup_refuses_foreign_and_private_altered_pairing() {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let foreign = runtime_with_test_schema();
    let result = create_entity_outcome(execution, &runtime, "original-registration");
    assert!(matches!(
        foreign
            .owner_component_services()
            .basis_port()
            .release_commit_result_snapshot(&result),
        Err(Denial::Snapshot(
            crate::visibility::RelationalSnapshotReleaseDenial::ForeignRuntime { .. }
        ))
    ));
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let independent = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    drop(basis);
    // This private negative fixture cannot be written through CommitResult's
    // public read-only outcome; both handles came from real owner admission.
    let altered = result.copy_with_altered_snapshot_for_test(independent.clone());
    let service = runtime.owner_component_services().basis_port();
    assert!(matches!(
        service.release_commit_result_snapshot(&altered),
        Err(Denial::RegistrationMismatch)
    ));
    assert_eq!(runtime.visibility.handles.published_count(), 1);
    assert_eq!(runtime.retention().active_snapshot_count(), 1);
    service.release_commit_result_snapshot(&result).unwrap();
    runtime.snapshots().release_snapshot(&independent).unwrap();
    runtime.prepare_close().unwrap().finish();
    foreign.prepare_close().unwrap().finish();
}

#[test]
fn result_snapshot_cleanup_registry_contention_preserves_exact_cell_and_competing_task() {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let mut runtime = runtime_with_test_schema();
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
    let handles = &runtime.visibility.handles;
    let original_root = handles
        .observe_published_root_for_test(result.snapshot.snapshot_id())
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
    let task = handles.while_published_registry_locked_for_test(1, || {
        let Progress::Pending {
            task,
            cause: Pending::Busy,
        } = task.advance(execution, &runtime)
        else {
            panic!("published contention must retain task without waiting")
        };
        assert!(!result.snapshot_release_observation().was_released());
        task
    });
    let Progress::Pending {
        task,
        cause: Pending::Advanced,
    } = task.advance(execution, &runtime)
    else {
        panic!("same task removes original registration")
    };
    assert_eq!(runtime.visibility.handles.published_count(), 0);
    assert_eq!(capacity.occupied_handles(), 0);
    let original = result.snapshot_release_observation();
    let competing = port.begin_commit_result_snapshot_cleanup(result.clone());
    let competing = original.while_removed_binding_locked_for_test(|| {
        let Progress::Pending {
            task: competing,
            cause: Pending::Busy,
        } = competing.advance(execution, &runtime)
        else {
            panic!("competing task must not replace or drop actual retirement")
        };
        competing
    });
    create_entity(execution, &runtime, "move-original-root-off-main");
    assert!(runtime
        .delete_branch(execution, &target)
        .unwrap()
        .deleted()
        .is_some());
    runtime.run_branch_root_reclamation_pass();
    assert!(
        original_root.upgrade().is_some(),
        "the exact original issued root survives head movement, branch removal and real reclamation while its binding awaits retirement"
    );
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
    assert_eq!(runtime.visibility.handles.published_count(), 0);
    assert_eq!(capacity.occupied_handles(), 0);
    runtime.run_branch_root_reclamation_pass();
    assert!(
        original_root.upgrade().is_none(),
        "the SAME original root is reclaimable only after actual binding retirement; no hidden test read/root lease pins it"
    );
    runtime.prepare_close().unwrap().finish();
}
