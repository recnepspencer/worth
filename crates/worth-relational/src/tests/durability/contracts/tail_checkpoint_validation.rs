use super::*;
use worth_foundational::{FoundationalBranchReferenceObservation, FoundationalBranchTarget};

#[test]
fn durability_contract_recovery_rejects_tail_target_without_immutable_artifact() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "checkpoint-basis");
    runtime.durability_authority().checkpoint().unwrap();
    let second = create_entity_outcome(execution, &runtime, "tail-commit");
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let tail = plan
        .tail_log
        .iter_mut()
        .map(|commit| commit.envelope_mut_for_test())
        .find(|envelope| envelope.commit.commit_id == second.commit.commit_id)
        .expect("tail contains the post-checkpoint commit");
    let checkpoint = tail
        .branch_cell_checkpoint
        .as_mut()
        .expect("tail envelope carries its exact pre-commit branch cell");
    let target = crate::branch::RelationalBranchTarget::new(
        checkpoint.runtime_instance_id,
        u64::MAX,
        u64::MAX,
        Vec::new(),
        crate::branch::RelationalBranchRootDescriptor::new([7; 32], [6; 32]),
    );
    checkpoint.observation = FoundationalBranchReferenceObservation::new(
        checkpoint.observation.branch_id().clone(),
        FoundationalBranchTarget::basis(target),
        checkpoint.observation.generation(),
    );

    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("tail admission must validate its target artifact");
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("recovery root"));
    assert!(error.detail.contains("is unavailable"));
    assert!(recovered.history().immutable_commit_count() <= 1);
    assert_ne!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_owned()))
            .map(|head| head.commit_id),
        Some(second.commit.commit_id)
    );
}
