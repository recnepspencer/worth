use super::*;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use worth_relational_contracts::internal_port::branch::target::RelationalBranchTargetOwnerPort as _;

#[test]
fn checkpoint_recovery_rejects_foreign_branch_target_substitution() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "lineage-seed");
    let feature = create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(execution, &runtime, "feature-only", feature.clone());
    create_entity_outcome(execution, &runtime, "main-only");
    medium.file_embedded_checkpoint(execution, &runtime);
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let checkpoint = plan.checkpoint.as_mut().unwrap();
    let foreign_target = checkpoint
        .branch_cells
        .iter()
        .find(|cell| cell.branch_id == feature)
        .unwrap()
        .observation
        .target()
        .clone();
    let main = checkpoint
        .branch_cells
        .iter_mut()
        .find(|cell| cell.branch_id.0 == "main")
        .unwrap();
    replace_checkpoint_target(main, foreign_target);

    let recovered_medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&recovered_medium);
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("foreign branch stream"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn tail_recovery_rejects_foreign_branch_target_substitution() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "tail-lineage-seed");
    let feature = create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-before-checkpoint",
        feature.clone(),
    );
    medium.file_embedded_checkpoint(execution, &runtime);
    let tail_commit = create_entity_outcome(execution, &runtime, "main-tail");
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let foreign_target = plan
        .checkpoint
        .as_ref()
        .unwrap()
        .branch_cells
        .iter()
        .find(|cell| cell.branch_id == feature)
        .unwrap()
        .observation
        .target()
        .clone();
    let tail_cell = (*plan
        .tail_log
        .iter_mut()
        .map(|commit| commit.readmitted_envelope_mut_for_test(execution))
        .find(|envelope| envelope.commit.commit_id == tail_commit.commit.commit_id)
        .unwrap()
        .owner_branch_cell_checkpoint_mut())
    .as_mut()
    .unwrap();
    replace_checkpoint_target(tail_cell, foreign_target);

    let recovered_medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&recovered_medium);
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("foreign branch stream"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

fn replace_checkpoint_target(
    checkpoint: &mut crate::branch::RelationalBranchCellCheckpoint,
    target: worth_foundational::FoundationalBranchTarget<crate::branch::RelationalBranchTarget>,
) {
    // The substitute is moved into the image's own runtime frame: a live
    // checkpoint cell and a canonical envelope's cell record different frames.
    let target = match target {
        worth_foundational::FoundationalBranchTarget::Basis(target) => {
            worth_foundational::FoundationalBranchTarget::basis(
                target.rebind_runtime_instance_id(checkpoint.runtime_instance_id),
            )
        }
        empty => empty,
    };
    checkpoint.observation = crate::branch::relational_branch_observation(
        checkpoint.runtime_instance_id,
        &checkpoint.branch_id.0,
        target,
        checkpoint.observation.generation(),
    )
    .unwrap();
}
