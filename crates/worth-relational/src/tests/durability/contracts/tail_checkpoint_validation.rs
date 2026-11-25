use worth_relational_durability::internal_port::durability::data::recovery_plan::RecoveryPlanFieldsFixturePort as _;
use worth_relational_durability::internal_port::durability::migration::readmitted_canonical_commit::ReadmittedCanonicalCommitOwnerPort as _;
use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use worth_foundational::{FoundationalBranchReferenceObservation, FoundationalBranchTarget};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use worth_relational_contracts::internal_port::branch::target::RelationalBranchTargetOwnerPort as _;

#[test]
fn durability_contract_recovery_rejects_tail_target_without_immutable_artifact() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "checkpoint-basis");
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

    let recovered_medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&recovered_medium);
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("tail admission must validate its target artifact");
    let error = crate::tests::support::expect_durability_failure(error);
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
