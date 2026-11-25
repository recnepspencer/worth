use super::*;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisOwnerPort as _;

fn fork(execution: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime, name: &str) {
    super::super::owner_service_phase3::fork_from_main(execution, runtime, name);
}

// The cursor's exact Arc custody is visible only inside the L6 runtime court.
#[test]
fn closed_original_and_target_cursor_holds_exact_private_custody_until_release() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let mut runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    fork(&execution, &runtime, "retained-original");
    fork(&execution, &runtime, "retained-target");
    let original_identity = runtime
        .branch_identity(&BranchId("retained-original".into()))
        .unwrap();
    let target_identity = runtime
        .branch_identity(&BranchId("retained-target".into()))
        .unwrap();
    let (_, original) = runtime.observe_branch(&original_identity).unwrap();
    let changed = update_entity_on_branch(
        &execution,
        &runtime,
        entity,
        "after",
        BranchId("retained-target".into()),
    );
    let (_, target) = runtime.observe_branch(&target_identity).unwrap();
    for identity in [&original_identity, &target_identity] {
        assert!(runtime
            .delete_branch(&execution, identity)
            .unwrap()
            .deleted()
            .is_some());
    }
    assert!(!original.is_current());
    for outcome in [&created, &changed] {
        release_test_commit_snapshot(&runtime, outcome);
    }
    let original_custody = original.owner_inner().downgrade();
    let target_custody = target.owner_inner().downgrade();
    let cursor = cursor(&runtime, execution, &original, &target, entity);
    drop(original);
    drop(target);
    assert!(original_custody.upgrade().is_some());
    assert!(target_custody.upgrade().is_some());
    assert_eq!(
        collect(&runtime, execution, cursor).0,
        [changed.commit.commit_id]
    );
    assert!(
        original_custody.upgrade().is_none(),
        "the last cursor releases original custody"
    );
    assert!(
        target_custody.upgrade().is_none(),
        "the last cursor releases target custody"
    );
}
