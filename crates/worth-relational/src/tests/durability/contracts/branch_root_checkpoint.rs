use super::*;

#[test]
fn checkpoint_recovery_rejects_missing_exact_branch_root_artifact() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "root-artifact-required");
    medium.file_embedded_checkpoint(execution, &runtime);
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    plan.checkpoint
        .as_mut()
        .expect("checkpoint is selected")
        .branch_roots
        .clear();

    let recovered_medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&recovered_medium);
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("global partition images cannot substitute for a branch root");
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(
        error.detail.contains("missing branch-root image"),
        "{error:?}"
    );
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn checkpoint_recovery_rejects_root_artifact_without_commit_envelope() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "root-envelope-binding");
    medium.file_embedded_checkpoint(execution, &runtime);
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let checkpoint = plan.checkpoint.as_mut().expect("checkpoint is selected");
    checkpoint.branch_roots[0].commit_id = crate::history::data::CommitId(u64::MAX);

    let recovered_medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&recovered_medium);
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("a root image cannot select an unrelated or missing envelope");
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("names missing commit envelope"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn checkpoint_recovery_rejects_duplicate_commit_envelopes_before_root_readmission() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "duplicate-envelope-root");
    medium.file_embedded_checkpoint(execution, &runtime);
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let checkpoint = plan.checkpoint.as_mut().expect("selected checkpoint");
    checkpoint.envelopes.push(checkpoint.envelopes[0].clone());

    let recovered_medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&recovered_medium);
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(
        error
            .detail
            .contains("reused by multiple canonical artifacts")
            || error
                .detail
                .contains("duplicate checkpoint commit envelope"),
        "{error:?}"
    );
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn checkpoint_recovery_rejects_duplicate_global_partition_images() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "duplicate-global-partition");
    medium.file_embedded_checkpoint(execution, &runtime);
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let checkpoint = plan.checkpoint.as_mut().expect("selected checkpoint");
    checkpoint
        .partition_images
        .push(checkpoint.partition_images[0].clone());

    let recovered_medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&recovered_medium);
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("duplicate partition image"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn checkpoint_recovery_rejects_duplicate_branch_root_partition_images() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "duplicate-root-partition");
    medium.file_embedded_checkpoint(execution, &runtime);
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let root = &mut plan.checkpoint.as_mut().unwrap().branch_roots[0];
    root.partition_images.push(root.partition_images[0].clone());

    let recovered_medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&recovered_medium);
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("duplicate partition image"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn branch_root_image_versions_refuse_before_restoration_and_current_recovers() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "current-root-format");
    medium.file_embedded_checkpoint(execution, &runtime);

    for found in [4, 5, 7] {
        let mut plan = medium
            .recovery_plan(
                execution,
                &runtime,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        let roots = &mut plan.checkpoint.as_mut().unwrap().branch_roots;
        assert!(
            !roots.is_empty(),
            "the real checkpoint must contain an owner root"
        );
        for root in roots {
            root.format_version = found;
        }
        let recovered_medium = OwnedTestDurableMedium::new();
        let mut recovered = durable_runtime_for_medium(&recovered_medium);
        let failure = recovered
            .durability_recovery()
            .recover(execution, plan)
            .unwrap_err();
        let failure = crate::tests::support::expect_durability_failure(failure);
        assert_eq!(
            failure.class,
            RecoveryFailureClass::UnsupportedBranchRootImageVersion {
                found,
                supported: 6,
            }
        );
        assert_eq!(recovered.history().immutable_commit_count(), 0);
    }

    let plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let recovered_medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&recovered_medium);
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(
        recovered.history().immutable_commit_count(),
        runtime.history().immutable_commit_count()
    );
}

#[path = "branch_root_checkpoint/foreign_targets.rs"]
mod foreign_targets;
