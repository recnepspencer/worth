//! Original roles are checked through the public Native source boundary.
use std::sync::Arc;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_relational::facade::{durability::*, runtime::RelationalRuntime};
use worth_testkit_relational::{
    aspect_storage::publications::release_test_commit_snapshot,
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, current},
    initial_input_source::*,
};
mod initial_tail;
mod plan_retry;
mod publication_fates;
mod wire;
fn initial(artifact: &RelationalCompleteSourceArtifact) -> serde_json::Value {
    let raw = artifact.as_bytes();
    let cp = u64::from_le_bytes(raw[28..36].try_into().unwrap()) as usize;
    let len = u64::from_le_bytes(raw[36..44].try_into().unwrap()) as usize;
    crate::source_wire_value::read(&raw[84 + cp + 60..84 + cp + len])
}
#[test]
fn complete_source_recovery_restores_original_inputs_for_another_complete_capture() {
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&policy);
    let (runtime, medium) = initialized(&request);
    let first = create_entity_outcome(&request, &runtime, "covered");
    release_test_commit_snapshot(&runtime, &first);
    let complete = capture(&runtime, &request);
    medium.file_checkpoint(complete.checkpoint().clone());
    let later = create_entity_outcome(&request, &runtime, "tail");
    release_test_commit_snapshot(&runtime, &later);
    let input = home_input(&medium, complete.source_artifact().clone(), &request).unwrap();
    drop((complete, runtime));
    let mut receiver = receiver(&medium);
    let plan = receiver
        .durability()
        .recovery_plan_from_source(
            &request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    receiver
        .durability_recovery()
        .recover_with_authority(&request, plan)
        .unwrap();
    let second = capture(&receiver, &request);
    assert_eq!(second.source_artifact().covered().0, 2);
    assert_eq!(second.source_artifact().initial_coordinate().0, 1);
    assert_eq!(
        initial(second.source_artifact())["entries"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    medium.file_checkpoint(second.checkpoint().clone());
    let input = home_input(&medium, second.source_artifact().clone(), &request).unwrap();
    assert!(receiver
        .durability()
        .recovery_plan_from_source(
            &request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification
        )
        .is_ok());
}
