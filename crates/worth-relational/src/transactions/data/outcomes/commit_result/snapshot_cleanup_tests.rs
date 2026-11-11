//! Explicit original-registration cleanup preserves independent read ownership.
use crate::facade::branch::{
    RelationalCommitResultSnapshotRelease as Outcome,
    RelationalCommitResultSnapshotReleaseDenial as Denial,
};
use crate::tests::support::*;
use worth_execution::ExecutionRequest;

#[test]
fn result_snapshot_cleanup_refuses_an_actual_admitted_runtime_copy() {
    use crate::facade::branch::{
        RelationalCommitResultSnapshotCleanupDenial as Denial,
        RelationalCommitResultSnapshotCleanupProgress as Progress,
    };
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
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
fn native_custody_correction_result_cleanup_is_explicit_idempotent_and_keeps_external_read() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let result = std::sync::Arc::new(create_entity_outcome(execution, &runtime, "owned-result"));
    let cloned = result.clone();
    let service = runtime.owner_component_services().basis_port();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let independent = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    drop(basis);
    assert!(matches!(
        service.release_commit_result_snapshot(&result).unwrap(),
        Outcome::Released(_)
    ));
    assert!(
        matches!(service.release_commit_result_snapshot(&cloned).unwrap(), Outcome::AlreadyReleased { snapshot_id } if snapshot_id == result.snapshot.snapshot_id())
    );
    assert_eq!(runtime.visibility.handles.published_count(), 0);
    assert_eq!(runtime.retention().active_snapshot_count(), 1);
    let refused = runtime.prepare_close().unwrap_err();
    assert!(
        matches!(refused.reason(), crate::runtime::RelationalRuntimeCloseDenial::ReadCustodyRetained(
            crate::facade::change_source::RelationalReadCustodyHolders::SnapshotsAndHistory {
                active_snapshots, detached_history,
            }
        ) if active_snapshots != 0 || detached_history)
    );
    let runtime = refused.into_runtime();
    runtime.snapshots().release_snapshot(&independent).unwrap();
    runtime.prepare_close().unwrap().finish();
    assert_eq!(
        cloned.commit, result.commit,
        "actual Arc results remain descriptive after explicit cleanup and close"
    );
    assert!(matches!(
        service.release_commit_result_snapshot(&result),
        Err(Denial::Owner(
            crate::branch::RelationalBranchBasisDenial::OwnerUnavailable
        ))
    ));
}

#[test]
fn native_custody_correction_result_release_cell_tracks_public_release_closeout_and_transfer() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
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
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let foreign = durable_runtime_with_test_schema();
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
    let mut altered = result.clone();
    altered.outcome.snapshot = independent.clone();
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
