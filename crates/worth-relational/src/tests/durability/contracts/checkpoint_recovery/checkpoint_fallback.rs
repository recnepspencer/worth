use worth_relational_durability::internal_port::durability::data::checkpoint_images::DurableCheckpointFieldsFixturePort as _;
use worth_relational_durability::internal_port::durability::data::recovery_plan::RecoveryPlanFieldsFixturePort as _;
use worth_relational_durability::internal_port::durability::migration::readmitted_canonical_commit::ReadmittedCanonicalCommitOwnerPort as _;
use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use worth_foundational::{FoundationalBranchReferenceObservation, FoundationalBranchTarget};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use worth_relational_contracts::internal_port::branch::target::RelationalBranchTargetOwnerPort as _;

#[test]
fn durability_contract_damaged_checkpoint_refuses_rather_than_shortening_history() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "first");
    capture_and_file_checkpoint(execution, &runtime, &medium);
    create_entity_outcome(execution, &runtime, "second");
    capture_and_file_checkpoint(execution, &runtime, &medium);
    medium.damage_checkpoint_payload();

    let recovered_medium = OwnedTestDurableMedium::new();
    let recovered = durable_runtime_for_medium(&recovered_medium);
    let refused = medium
        .recovery_plan(
            execution,
            &recovered,
            RecoveryMode::NormalRecoveryVerification,
        )
        .expect_err("a damaged checkpoint is refused, never skipped");

    assert_eq!(
        expect_durability_failure(refused).class,
        RecoveryFailureClass::CorruptCheckpoint
    );
    assert!(recovered
        .history()
        .branch_head(&BranchId("main".to_string()))
        .is_none());
}

#[test]
fn durability_contract_recovery_fails_closed_without_exact_branch_cells() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "branch-cell-required");
    capture_and_file_checkpoint(execution, &runtime, &medium);
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    plan.checkpoint
        .as_mut()
        .expect("checkpoint is selected")
        .branch_cells_mut_for_test()
        .clear();

    let recovered_medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&recovered_medium);
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("recovery must not synthesize branch cells from legacy heads");
    let error = crate::tests::support::expect_durability_failure(error);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error
        .detail
        .contains("durable checkpoint omitted exact branch-cell state"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
    assert!(recovered
        .history()
        .branch_head(&BranchId("main".to_owned()))
        .is_none());
}

#[test]
fn durability_contract_recovery_rejects_branch_cell_target_without_catalog_artifact() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "branch-cell-artifact-required");
    capture_and_file_checkpoint(execution, &runtime, &medium);
    let mut plan = medium
        .expanded_recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let checkpoint = plan.checkpoint.as_mut().expect("checkpoint is selected");
    let main_cell = checkpoint
        .branch_cells_mut_for_test()
        .iter_mut()
        .find(|cell| cell.branch_id == BranchId("main".to_owned()))
        .expect("main branch cell is checkpointed");
    let target = crate::branch::RelationalBranchTarget::new(
        main_cell.runtime_instance_id,
        u64::MAX,
        u64::MAX,
        Vec::new(),
        crate::branch::RelationalBranchRootDescriptor::new([9; 32], [8; 32]),
    );
    main_cell.observation = FoundationalBranchReferenceObservation::new(
        main_cell.observation.branch_id().clone(),
        FoundationalBranchTarget::basis(target),
        main_cell.observation.generation(),
    );

    let recovered_medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&recovered_medium);
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("recovery must reject a branch target with no immutable artifact");
    let error = crate::tests::support::expect_durability_failure(error);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(
        error.detail.contains("references missing commit artifact"),
        "{error:?}"
    );
    assert_eq!(recovered.history().immutable_commit_count(), 0);
    assert!(recovered
        .history()
        .branch_head(&BranchId("main".to_owned()))
        .is_none());
}

#[test]
fn durability_contract_recovery_rejects_tail_checkpoint_drift_before_mutation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    let _first = create_entity_outcome(execution, &runtime, "checkpoint-basis");
    capture_and_file_checkpoint(execution, &runtime, &medium);
    let second = create_entity_outcome(execution, &runtime, "tail-commit");
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let tail = plan
        .tail_log_mut_for_test()
        .iter_mut()
        .map(|commit| commit.readmitted_envelope_mut_for_test(execution))
        .find(|envelope| envelope.commit.commit_id == second.commit.commit_id)
        .expect("tail contains the post-checkpoint commit");
    let checkpoint = (*tail.owner_branch_cell_checkpoint_mut())
        .as_mut()
        .expect("tail envelope carries its exact pre-commit branch cell");
    checkpoint.truth_version = crate::branch::RelationalBranchVersion::new(
        checkpoint.truth_version.as_u64().saturating_add(1),
    );

    let recovered_medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&recovered_medium);
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("recovery must reject a mismatching existing branch checkpoint");
    let error = crate::tests::support::expect_durability_failure(error);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error
        .detail
        .contains("recovery branch-cell state conflicts"));
    assert!(recovered.history().immutable_commit_count() <= 1);
    assert_ne!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_owned()))
            .map(|head| head.commit_id),
        Some(second.commit.commit_id)
    );
}
