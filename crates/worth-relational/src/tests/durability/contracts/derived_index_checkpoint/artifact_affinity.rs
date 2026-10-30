use super::*;
use crate::indexes::data::DerivedIndexArtifacts;

#[test]
fn legacy_index_affinity_is_checked_before_replacing_a_live_recovery_target() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let source = durable_runtime_with_test_schema();
    let committed = create_entity_outcome(execution, &source, "legacy-affinity-source");
    let index = name_index(&source);
    build(execution, &source, index.index_id, &committed);
    release_test_commit_snapshot(&source, &committed);
    crate::tests::support::checkpoint_durably(execution, &source).unwrap();
    let mut lawful = crate::tests::support::recovery_plan_for(
        execution,
        &source,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let checkpoint = lawful.checkpoint.as_mut().unwrap();
    let generations = checkpoint
        .derived_index_checkpoint
        .take()
        .unwrap()
        .readmit()
        .unwrap();
    checkpoint.derived_index_checkpoint_format = 0;
    checkpoint.derived_index_artifacts = DerivedIndexArtifacts::new(generations.clone());
    let mut corrupt = lawful.clone();
    let mut malformed = generations;
    malformed[0].applicability.branch_id = BranchId("foreign-legacy-branch".into());
    corrupt.checkpoint.as_mut().unwrap().derived_index_artifacts =
        DerivedIndexArtifacts::new(malformed);
    let mut target = durable_runtime_with_test_schema();
    let sentinel = create_entity_outcome(execution, &target, "preserved-target");
    release_test_commit_snapshot(&target, &sentinel);
    let identity = target.branch_identity(&BranchId("main".into())).unwrap();
    let before = target.observe_branch(&identity).unwrap().0;
    let error = target
        .durability_recovery()
        .recover(execution, corrupt)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("branch affinity"), "{error:?}");
    assert_eq!(target.observe_branch(&identity).unwrap().0, before);
    let mut control = durable_runtime_with_test_schema();
    control
        .durability_recovery()
        .recover(execution, lawful)
        .unwrap();
}

#[test]
fn envelope_index_affinity_requires_admission_before_installation() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    let committed = create_entity_outcome(execution, &runtime, "envelope-affinity");
    let index = name_index(&runtime);
    let id = build(execution, &runtime, index.index_id, &committed);
    let actual = runtime.indexes.generation(id).unwrap();
    let mut envelope = committed.envelope().clone();
    envelope.derived_index_artifacts = DerivedIndexArtifacts::new(vec![actual.as_ref().clone()]);
    crate::durability::derived_index_artifacts::admit_envelope_derived_index_artifacts(&envelope)
        .unwrap();
    let mut malformed = actual.as_ref().clone();
    malformed.applicability.branch_id = BranchId("foreign-envelope-branch".into());
    envelope.derived_index_artifacts = DerivedIndexArtifacts::new(vec![malformed]);
    let error = crate::durability::derived_index_artifacts::admit_envelope_derived_index_artifacts(
        &envelope,
    )
    .err()
    .expect("transported affinity mismatch must not mint installation admission");
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("branch affinity"));
    assert_eq!(
        runtime.indexes.generation(id).unwrap().as_ref(),
        actual.as_ref()
    );
}
