use super::*;
use crate::branch::PositionedBranchTransition;
use crate::facade::durability::{DurabilityError, RecoveryFailureClass, RecoveryVerificationMode};
use crate::facade::publication::PatchStreamPosition;

fn refusal(execution: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime) -> DurabilityError {
    let mut recovered = durable_runtime_with_test_schema();
    let result = medium_recovery_plan(
        execution,
        runtime,
        &mut recovered,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .and_then(|plan| recovered.durability_recovery().recover(execution, plan));
    expect_durability_failure(result.expect_err("corrupt history must refuse before service"))
}

#[test]
fn a_damaged_transition_frame_is_refused_at_the_medium_read() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "source");
    fork(execution, &runtime, "target");
    let medium = attached_test_medium(&runtime);
    let bytes = medium.transition_bytes(Sequence(1)).unwrap();
    medium.flip_transition_byte(Sequence(1), bytes.len() - 1);
    let error = medium.recovery_source().unwrap_err();
    let crate::tests::support::TestRecoverySourceRefusal::Transition(refusal) = error else {
        panic!("expected transition frame refusal: {error:?}");
    };
    assert!(matches!(refusal, crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifactFrameRefusal::DigestMismatch { .. }));
}

#[test]
fn a_sequence_gap_and_invalid_follows_positions_are_refused() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    for defect in ["gap", "regression", "beyond-head"] {
        let runtime = durable_runtime_with_test_schema();
        create_entity(execution, &runtime, "first");
        fork(execution, &runtime, "first-fork");
        create_entity(execution, &runtime, "second");
        fork(execution, &runtime, "second-fork");
        let pristine = reopen(execution, &runtime);
        assert_eq!(artifacts(&pristine), artifacts(&runtime));
        let medium = attached_test_medium(&runtime);
        if defect == "gap" {
            medium.remove_transition(Sequence(1));
        } else {
            let held = medium.transition_bytes(Sequence(2)).unwrap();
            let transition = Artifact::readmit(&held, execution).unwrap();
            let follows = if defect == "regression" { 0 } else { 3 };
            let changed = PositionedBranchTransition::admit(
                Sequence(2),
                PatchStreamPosition(follows),
                transition.transition().clone(),
            );
            let artifact =
                Artifact::from_positioned(&changed, ExecutionAllocationPolicy::SystemAllocation)
                    .unwrap();
            medium.replace_transition(Sequence(2), artifact.as_bytes().into());
        }
        assert_eq!(
            refusal(execution, &runtime).class,
            RecoveryFailureClass::NonContiguousBranchTransitionTail
        );
    }
}

#[test]
fn a_commit_carrying_an_unknown_branch_cell_cannot_replace_its_missing_fork() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "source");
    fork(execution, &runtime, "target");
    create_entity_outcome_on_branch(execution, &runtime, "target", BranchId("target".into()));
    let pristine = reopen(execution, &runtime);
    assert_eq!(artifacts(&pristine), artifacts(&runtime));
    attached_test_medium(&runtime).remove_transition(Sequence(1));
    let error = refusal(execution, &runtime);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(
        error
            .detail
            .contains("which no checkpoint or durable branch transition created"),
        "{error:?}"
    );
}
