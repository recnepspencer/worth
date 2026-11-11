//! Digest-valid checkpoint index artifacts must still carry complete, current meaning.
use super::*;

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
