use crate::tests::support::*;

#[test]
fn current_branch_snapshot_never_substitutes_another_branch_head() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "base");
    let main = BranchId("main".to_owned());
    let feature = BranchId("feature".to_owned());
    runtime
        .history_authority()
        .fork_branch_from(feature.clone(), &main)
        .unwrap();
    update_entity_on_branch(execution, &runtime, entity, "feature", feature.clone());

    let main_snapshot = snapshot_for_owner_branch(&runtime, &main);
    let feature_snapshot = snapshot_for_owner_branch(&runtime, &feature);

    assert_eq!(main_snapshot.branch_id, main);
    assert_eq!(feature_snapshot.branch_id, feature);
    assert_eq!(
        main_snapshot.version_id,
        runtime.history().branch_head(&main).unwrap().version_id
    );
    assert_eq!(
        feature_snapshot.version_id,
        runtime.history().branch_head(&feature).unwrap().version_id
    );
    assert_ne!(main_snapshot.snapshot_id, feature_snapshot.snapshot_id);
    assert!(runtime.snapshots().release_snapshot(&main_snapshot).is_ok());
    assert!(runtime
        .snapshots()
        .release_snapshot(&feature_snapshot)
        .is_ok());
    assert!(runtime
        .branch_identity(&BranchId("missing".to_owned()))
        .is_err());
}
