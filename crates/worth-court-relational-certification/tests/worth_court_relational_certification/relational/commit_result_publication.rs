//! Original commit result cleanup is observable at the owner service boundary.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::{
        RelationalBranchBasisDenial, RelationalCommitResultSnapshotRelease as Outcome,
        RelationalCommitResultSnapshotReleaseDenial as Denial,
    },
    change_source::RelationalReadCustodyHolders,
    runtime::{RelationalRetainedCommitSnapshotDenialKind, RelationalRuntimeCloseDenial},
};
use worth_testkit_relational::{
    durable_publication::{create_entity_outcome, memory_runtime},
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn native_custody_correction_result_cleanup_is_explicit_idempotent_and_keeps_external_read() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
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
    assert_eq!(
        runtime
            .snapshots()
            .retained_snapshot_for_commit(runtime.runtime_instance_id(), &result.commit)
            .unwrap_err()
            .kind(),
        RelationalRetainedCommitSnapshotDenialKind::SnapshotNotRetained,
    );
    assert_eq!(runtime.retention().active_snapshot_count(), 1);
    let refused = runtime.prepare_close().unwrap_err();
    assert!(
        matches!(refused.reason(), RelationalRuntimeCloseDenial::ReadCustodyRetained(
            RelationalReadCustodyHolders::SnapshotsAndHistory {
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
        Err(Denial::Owner(RelationalBranchBasisDenial::OwnerUnavailable))
    ));
}
