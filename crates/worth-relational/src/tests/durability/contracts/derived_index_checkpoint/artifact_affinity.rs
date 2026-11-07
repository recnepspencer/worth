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
        .readmit(&crate::storage::substrate::maintenance::ReconstructionLane::oracle())
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

use crate::indexes::data::{
    DerivedIndexEntries, RelationJoinDefinition, RelationJoinLeg, RelationJoinSharedEndpoint,
};

#[test]
fn digest_valid_join_checkpoint_requires_complete_rows_and_leg_state() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    let left = create_entity(execution, &runtime, "join-left");
    let hub = create_entity(execution, &runtime, "join-hub");
    let right = create_entity(execution, &runtime, "join-right");
    create_relation(execution, &runtime, left, hub, "left-leg");
    let committed = create_relation_outcome(execution, &runtime, hub, right, "right-leg");
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "checkpoint-join".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::RelationJoin(RelationJoinDefinition::new(
                RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Target, KindId(1)),
                RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Source, KindId(1)),
                KindId(1),
            )),
        })
        .unwrap();
    build(execution, &runtime, index.index_id, &committed);
    release_test_commit_snapshot(&runtime, &committed);
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let lawful = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut control = durable_runtime_with_test_schema();
    control
        .durability_recovery()
        .recover(execution, lawful.clone())
        .unwrap();
    for missing_rows in [true, false] {
        let mut malformed = lawful.clone();
        let checkpoint = malformed.checkpoint.as_mut().unwrap();
        let artifact = checkpoint.derived_index_checkpoint.as_ref().unwrap();
        let mut generations = artifact
            .readmit(&crate::storage::substrate::maintenance::ReconstructionLane::oracle())
            .unwrap();
        let generation = generations
            .iter_mut()
            .find(|generation| generation.index_id == index.index_id)
            .unwrap();
        let DerivedIndexEntries::RelationJoin(rows) = &mut generation.entries else {
            panic!("join payload")
        };
        assert!(!rows.is_empty());
        if missing_rows {
            rows.clear();
        } else {
            *rows = (**rows).clone().into();
        }
        // Recompute the digest: the trust boundary must check meaning as well as bytes.
        checkpoint.derived_index_checkpoint = Some(
            crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::capture(
                &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
                generations.into_iter().map(std::sync::Arc::new).collect(),
                artifact.bases().to_vec(),
            )
            .unwrap(),
        );
        let mut target = durable_runtime_with_test_schema();
        let error = target
            .durability_recovery()
            .recover(execution, malformed)
            .unwrap_err();
        let error = crate::tests::support::expect_durability_failure(error);
        assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
        assert!(error.detail.contains("join leg state"), "{error:?}");
    }
}

#[test]
fn digest_valid_basis_binding_cannot_carry_an_obsolete_generation_across_an_edit() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "basis-before");
    let entity = changed_entities(&first)[0];
    let index = name_index(&runtime);
    let prior = build(execution, &runtime, index.index_id, &first);
    let prior = runtime.indexes.generation(prior).unwrap();
    let second = update_entity(execution, &runtime, entity, "basis-after");
    build(execution, &runtime, index.index_id, &second);
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let checkpoint = plan.checkpoint.as_mut().unwrap();
    let artifact = checkpoint.derived_index_checkpoint.as_ref().unwrap();
    let mut generations: Vec<_> = artifact
        .readmit(&crate::storage::substrate::maintenance::ReconstructionLane::oracle())
        .unwrap()
        .into_iter()
        .map(std::sync::Arc::new)
        .collect();
    generations.push(prior.clone());
    let mut bases = artifact.bases().to_vec();
    let current = bases
        .iter_mut()
        .find(|basis| basis.commit == second.commit.commit_id)
        .unwrap();
    current
        .bindings
        .iter_mut()
        .find(|(id, _)| *id == index.index_id)
        .unwrap()
        .1 = prior.generation_id;
    checkpoint.derived_index_checkpoint = Some(
        crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::capture(
            &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
            generations,
            bases,
        )
        .unwrap(),
    );
    let mut target = durable_runtime_with_test_schema();
    let error = target
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("authoritative root"), "{error:?}");
}
