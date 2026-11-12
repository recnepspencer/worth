use super::branch_history_publications::{create_entity, create_entity_outcome_on_branch, fork};
use super::branch_history_recovery::current_head;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::durable_runtime;

use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_relational::facade::branch::{
    RelationalBranchLifecyclePosture as Posture, RelationalBranchTransitionStop as Stop,
    RelationalForkDenial,
};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
#[test]
fn a_durable_fork_without_a_subsequent_commit_reopens_and_commits() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "source");
    fork(execution, &runtime, "target");
    let recovered =
        super::branch_history_recovery::recover(execution, &medium, durable_runtime(&medium)).1;
    create_entity_outcome_on_branch(
        execution,
        &recovered,
        "after-crash",
        BranchId("target".into()),
    );
    let again =
        super::branch_history_recovery::recover(execution, &medium, durable_runtime(&medium)).1;
    assert_eq!(
        current_head(&again, &BranchId("target".into())),
        current_head(&recovered, &BranchId("target".into()))
    );
}

#[test]
fn a_fork_covered_by_a_checkpoint_after_its_source_moves_accepts_a_target_tail() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "source");
    fork(execution, &runtime, "target");
    create_entity(execution, &runtime, "source-advanced");
    super::branch_history_recovery::checkpoint(execution, &runtime, &medium);
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "target-tail",
        BranchId("target".into()),
    );
    let recovered =
        super::branch_history_recovery::recover(execution, &medium, durable_runtime(&medium)).1;
    assert_eq!(
        current_head(&recovered, &BranchId("target".into())),
        current_head(&runtime, &BranchId("target".into()))
    );
}

#[test]
fn a_fork_in_the_transition_tail_is_merged_before_the_target_commit() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "checkpoint-source");
    super::branch_history_recovery::checkpoint(execution, &runtime, &medium);
    fork(execution, &runtime, "target");
    create_entity(execution, &runtime, "source-tail");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "target-tail",
        BranchId("target".into()),
    );
    let recovered =
        super::branch_history_recovery::recover(execution, &medium, durable_runtime(&medium)).1;
    assert_eq!(
        current_head(&recovered, &BranchId("target".into())),
        current_head(&runtime, &BranchId("target".into()))
    );
    assert_eq!(artifacts(&recovered), artifacts(&runtime));
}

#[test]
fn lifecycle_postures_and_retired_names_survive_checkpoints_and_crashes() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let medium = TestDurableMedium::new();
    let mut runtime = durable_runtime(&medium);
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
            super::branch_history_recovery::checkpoint(execution, &runtime, &medium);
        }
        let recovered =
            super::branch_history_recovery::recover(execution, &medium, durable_runtime(&medium)).1;
        for (name, posture) in [
            ("archived", Posture::Archived),
            ("deleting", Posture::Deleting),
        ] {
            assert_eq!(
                recovered
                    .branch_reference_state(&BranchId(name.into()))
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
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let history = || {
        let medium = TestDurableMedium::new();
        let mut runtime = durable_runtime(&medium);
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

fn artifacts(runtime: &RelationalRuntime) -> Vec<Vec<u8>> {
    runtime
        .durability()
        .branch_transition_artifacts(ExecutionAllocationPolicy::SystemAllocation)
        .map(|artifact| artifact.unwrap().as_bytes().to_vec())
        .collect()
}
