use super::*;
use crate::branch::{RelationalBranchLifecyclePosture as Posture, RelationalForkDenial};
use crate::facade::mvcc::RelationalTransactionIntent;

#[test]
fn a_durable_fork_without_a_subsequent_commit_reopens_and_commits() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "source");
    fork(execution, &runtime, "target");
    let recovered = reopen(execution, &runtime);
    create_entity_outcome_on_branch(
        execution,
        &recovered,
        "after-crash",
        BranchId("target".into()),
    );
    let again = reopen(execution, &recovered);
    assert_eq!(
        again.history().branch_head(&BranchId("target".into())),
        recovered.history().branch_head(&BranchId("target".into()))
    );
}

#[test]
fn a_fork_covered_by_a_checkpoint_after_its_source_moves_accepts_a_target_tail() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "source");
    fork(execution, &runtime, "target");
    create_entity(execution, &runtime, "source-advanced");
    checkpoint_into_medium(execution, &runtime);
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "target-tail",
        BranchId("target".into()),
    );
    let recovered = reopen(execution, &runtime);
    assert_eq!(
        recovered.history().branch_head(&BranchId("target".into())),
        runtime.history().branch_head(&BranchId("target".into()))
    );
}

#[test]
fn a_fork_in_the_transition_tail_is_merged_before_the_target_commit() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "checkpoint-source");
    checkpoint_into_medium(execution, &runtime);
    fork(execution, &runtime, "target");
    create_entity(execution, &runtime, "source-tail");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "target-tail",
        BranchId("target".into()),
    );
    let recovered = reopen(execution, &runtime);
    assert_eq!(
        recovered.history().branch_head(&BranchId("target".into())),
        runtime.history().branch_head(&BranchId("target".into()))
    );
    assert_eq!(artifacts(&recovered), artifacts(&runtime));
}

#[test]
fn lifecycle_postures_and_retired_names_survive_checkpoints_and_crashes() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let mut runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "source");
    for name in ["archived", "deleting", "deleted"] {
        fork(execution, &runtime, name);
    }
    let archived = runtime
        .branch_identity(&BranchId("archived".into()))
        .unwrap();
    runtime.archive_branch(execution, &archived).unwrap();
    let deleting = runtime
        .branch_identity(&BranchId("deleting".into()))
        .unwrap();
    let (_, basis) = runtime.observe_branch(&deleting).unwrap();
    let transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    runtime
        .owner_component_services()
        .lifecycle_port()
        .delete_branch(execution, &deleting)
        .unwrap();
    drop(transaction);
    drop(basis);
    let deleted = runtime
        .branch_identity(&BranchId("deleted".into()))
        .unwrap();
    runtime.delete_branch(execution, &deleted).unwrap();
    for checkpoint in [false, true] {
        if checkpoint {
            checkpoint_into_medium(execution, &runtime);
        }
        let recovered = reopen(execution, &runtime);
        for (name, posture) in [
            ("archived", Posture::Archived),
            ("deleting", Posture::Deleting),
        ] {
            assert_eq!(
                recovered
                    .history
                    .branch_cell(&BranchId(name.into()))
                    .unwrap()
                    .lifecycle_posture(),
                posture
            );
        }
        assert!(recovered
            .branch_identity(&BranchId("deleted".into()))
            .is_err());
        for name in ["deleting", "deleted"] {
            let (_, basis) = recovered
                .observe_fork_source(&BranchId("main".into()))
                .unwrap();
            assert!(matches!(
                recovered.fork_branch(execution, BranchId(name.into()), basis),
                Err(Stop::Denied(RelationalForkDenial::RetiredTarget))
            ));
        }
    }
}

#[test]
fn independent_runtimes_encode_byte_identical_transition_histories() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let history = || {
        let mut runtime = durable_runtime_with_test_schema();
        create_entity(execution, &runtime, "source");
        fork(execution, &runtime, "target");
        let identity = runtime.branch_identity(&BranchId("target".into())).unwrap();
        runtime.archive_branch(execution, &identity).unwrap();
        runtime.delete_branch(execution, &identity).unwrap();
        fork(execution, &runtime, "pending");
        let identity = runtime
            .branch_identity(&BranchId("pending".into()))
            .unwrap();
        let (_, basis) = runtime.observe_branch(&identity).unwrap();
        let transaction = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        runtime
            .owner_component_services()
            .lifecycle_port()
            .delete_branch(execution, &identity)
            .unwrap();
        drop(transaction);
        artifacts(&runtime)
    };
    let first = history();
    assert_eq!(first.len(), 5);
    assert_eq!(first, history());
}
