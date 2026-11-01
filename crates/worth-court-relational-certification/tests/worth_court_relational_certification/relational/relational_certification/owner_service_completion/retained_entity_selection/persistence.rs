//! Unsupported persistence refuses through the real medium and recovery doors.
use super::super::unaccessed_medium::UnaccessedMedium;
use super::occurrences::{fixture, perform, prepare};
use super::*;
use worth_relational::facade::{
    durability::*, mvcc::RelationalRetainedEntitySelectionDenial as Denial,
    runtime::RelationalRuntimeApi,
};

#[test]
fn selection_raw_replay_refuses_before_installing_its_effects() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = fixture().build_runtime();
    let old = create(execution, &runtime, "model", "3");
    let model = changed_entities(&old)[0];
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let current = update(execution, &runtime, [(model, "5")]);
    let selected = perform(
        execution,
        &runtime,
        prepare(execution, &runtime, &source, model),
    );
    let plan = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut restored = fixture().build_runtime();
    let RelationalRecoveryStop::Durability(cause) = restored
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err()
    else {
        panic!("raw selection replay must return its specific unsupported profile")
    };
    assert_eq!(
        cause.class,
        RecoveryFailureClass::UnsupportedRetainedEntitySelectionReplay
    );
    assert_ne!(
        restored
            .admit_branch_basis(&restored.main_branch_identity())
            .unwrap()
            .observation()
            .commit_id(),
        Some(selected.commit.commit_id)
    );
    for commit in [&old, &current, &selected] {
        release_test_commit_snapshot(&runtime, commit);
    }
}

#[test]
fn attached_medium_refuses_selection_before_any_medium_access() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = fixture().build_runtime();
    let old = create(execution, &runtime, "model", "3");
    let model = changed_entities(&old)[0];
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let durable = RelationalRuntimeApi::builder()
        .schema_registry(fixture().build_registry())
        .build_durable(Box::new(UnaccessedMedium))
        .unwrap();
    let transaction = begin_main_transaction(&durable);
    assert_eq!(
        durable
            .owner_component_services()
            .preparation_port()
            .admit_retained_entity_selection(transaction, &source, model)
            .unwrap_err(),
        Denial::DurabilityUnsupported
    );
    assert_eq!(
        durable
            .admit_branch_basis(&durable.main_branch_identity())
            .unwrap()
            .observation()
            .commit_id(),
        None
    );
    let lawful = perform(
        execution,
        &runtime,
        prepare(execution, &runtime, &source, model),
    );
    assert_eq!(
        values(execution, &runtime, &lawful, [model, model]),
        ["3", "3"].map(string_aspect_value)
    );
    for commit in [&old, &lawful] {
        release_test_commit_snapshot(&runtime, commit);
    }
}
