//! Digest-valid checkpoint index artifacts must still carry complete, current meaning.
use super::*;
#[cfg(test)]
use crate::durability::canonical_commit_artifact::checkpoint_body::copy_original_for_test;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use crate::indexes::data::{
    DerivedIndexArtifacts, DerivedIndexEntries, RelationJoinDefinition, RelationJoinLeg,
    RelationJoinSharedEndpoint,
};
use worth_relational_contracts::internal_port::indexes::data::entry_map::DerivedIndexEntryMapOwnerPort as _;

#[test]
fn digest_valid_join_checkpoint_requires_complete_rows_and_leg_state() {
    crate::branch::root::capture::with_reconstruction_fixture(|fixture_lane| {
        let request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&request);
        let medium = OwnedTestDurableMedium::new();
        let runtime = durable_runtime_for_medium(&medium);
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
        medium.file_embedded_checkpoint(execution, &runtime);
        let lawful = medium
            .recovery_plan(
                execution,
                &runtime,
                RecoveryMode::NormalRecoveryVerification,
            )
            .unwrap();
        let control_medium = OwnedTestDurableMedium::new();
        let mut control = durable_runtime_for_medium(&control_medium);
        control
            .durability_recovery()
            .recover(execution, lawful.clone())
            .unwrap();
        for missing_rows in [true, false] {
            let mut malformed = lawful.clone();
            let checkpoint = malformed.checkpoint.as_mut().unwrap();
            let artifact = checkpoint.derived_index_checkpoint.as_ref().unwrap();
            let mut generations = artifact.readmit(fixture_lane).unwrap();
            let generation = generations
                .iter_mut()
                .find(|generation| generation.index_id == index.index_id)
                .unwrap();
            let DerivedIndexEntries::RelationJoin(rows) = &mut generation.entries else {
                panic!("join payload")
            };
            assert!(!rows.is_empty());
            if missing_rows {
                rows.clear_index_entries_for_test();
            } else {
                *rows = (**rows).clone().into();
            }
            // Recompute the digest: the trust boundary must check meaning as well as bytes.
            checkpoint.derived_index_checkpoint = Some(
            crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::capture(
                fixture_lane,
                generations.into_iter().map(std::sync::Arc::new).collect(),
                artifact.bases().to_vec(),
            )
            .unwrap(),
        );
            let target_medium = OwnedTestDurableMedium::new();
            let mut target = durable_runtime_for_medium(&target_medium);
            let error = target
                .durability_recovery()
                .recover(execution, malformed)
                .unwrap_err();
            let error = crate::tests::support::expect_durability_failure(error);
            assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
            assert!(error.detail.contains("join leg state"), "{error:?}");
        }
    })
}

#[test]
fn digest_valid_basis_binding_cannot_carry_an_obsolete_generation_across_an_edit() {
    crate::branch::root::capture::with_reconstruction_fixture(|fixture_lane| {
        let request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&request);
        let medium = OwnedTestDurableMedium::new();
        let runtime = durable_runtime_for_medium(&medium);
        let first = create_entity_outcome(execution, &runtime, "basis-before");
        let entity = changed_entities(&first)[0];
        let index = name_index(&runtime);
        let prior = build(execution, &runtime, index.index_id, &first);
        let prior = runtime.indexes.generation(prior).unwrap();
        let second = update_entity(execution, &runtime, entity, "basis-after");
        build(execution, &runtime, index.index_id, &second);
        release_test_commit_snapshot(&runtime, &first);
        release_test_commit_snapshot(&runtime, &second);
        medium.file_embedded_checkpoint(execution, &runtime);
        let mut plan = medium
            .recovery_plan(
                execution,
                &runtime,
                RecoveryMode::NormalRecoveryVerification,
            )
            .unwrap();
        let checkpoint = plan.checkpoint.as_mut().unwrap();
        let artifact = checkpoint.derived_index_checkpoint.as_ref().unwrap();
        let mut generations: Vec<_> = artifact
            .readmit(fixture_lane)
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
                fixture_lane,
                generations,
                bases,
            )
            .unwrap(),
        );
        let target_medium = OwnedTestDurableMedium::new();
        let mut target = durable_runtime_for_medium(&target_medium);
        let error = target
            .durability_recovery()
            .recover(execution, plan)
            .unwrap_err();
        let error = crate::tests::support::expect_durability_failure(error);
        assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
        assert!(error.detail.contains("authoritative root"), "{error:?}");
    })
}

#[test]
fn legacy_index_affinity_is_checked_before_replacing_a_live_recovery_target() {
    crate::branch::root::capture::with_reconstruction_fixture(|fixture_lane| {
        let request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&request);
        let source_medium = OwnedTestDurableMedium::new();
        let source = durable_runtime_for_medium(&source_medium);
        let committed = create_entity_outcome(execution, &source, "legacy-affinity-source");
        let index = name_index(&source);
        build(execution, &source, index.index_id, &committed);
        release_test_commit_snapshot(&source, &committed);
        source_medium.file_embedded_checkpoint(execution, &source);
        let mut lawful = source_medium
            .recovery_plan(execution, &source, RecoveryMode::NormalRecoveryVerification)
            .unwrap();
        let checkpoint = lawful.checkpoint.as_mut().unwrap();
        let generations = checkpoint
            .derived_index_checkpoint
            .take()
            .unwrap()
            .readmit(fixture_lane)
            .unwrap();
        checkpoint.derived_index_checkpoint_format = 0;
        checkpoint.derived_index_artifacts = DerivedIndexArtifacts::new(generations.clone());
        let mut corrupt = lawful.clone();
        let mut malformed = generations;
        malformed[0].applicability.branch_id = BranchId("foreign-legacy-branch".into());
        corrupt.checkpoint.as_mut().unwrap().derived_index_artifacts =
            DerivedIndexArtifacts::new(malformed);
        let target_medium = OwnedTestDurableMedium::new();
        let mut target = durable_runtime_for_medium(&target_medium);
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
        let control_medium = OwnedTestDurableMedium::new();
        let mut control = durable_runtime_for_medium(&control_medium);
        control
            .durability_recovery()
            .recover(execution, lawful)
            .unwrap();
    })
}

#[test]
fn envelope_index_affinity_requires_admission_before_installation() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    let committed = create_entity_outcome(execution, &runtime, "envelope-affinity");
    let index = name_index(&runtime);
    let id = build(execution, &runtime, index.index_id, &committed);
    let actual = runtime.indexes.generation(id).unwrap();
    let mut envelope = copy_original_for_test(&committed.envelope(), execution);
    envelope.body_mut_for_test().derived_index_artifacts =
        DerivedIndexArtifacts::new(vec![actual.as_ref().clone()]);
    crate::durability::derived_index_artifacts::admit_envelope_derived_index_artifacts(&envelope)
        .unwrap();
    let mut malformed = actual.as_ref().clone();
    malformed.applicability.branch_id = BranchId("foreign-envelope-branch".into());
    envelope.body_mut_for_test().derived_index_artifacts =
        DerivedIndexArtifacts::new(vec![malformed]);
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

#[test]
fn residency_cold_index_frontier_prevents_reissue_before_any_image_demand() {
    use std::sync::{atomic::Ordering, Arc};
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for (bulk, admitted) in [(false, false), (true, false), (false, true), (true, true)] {
        let producer =
            runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
        let index = name_index(&producer).index_id;
        let committed = create_entity_outcome(execution, &producer, "frontier-index-value");
        let generation_id = build(execution, &producer, index, &committed);
        let generation = producer.indexes.generation(generation_id).unwrap();
        let (runtime, source) = reopen_cold_index_members(execution, &producer);
        let owner = &runtime.indexes;
        let reserved = if bulk {
            owner.reserve_generation_ids(1).unwrap()[0]
        } else {
            crate::indexes::data::DerivedIndexGenerationId(owner.next_generation_id().unwrap())
        };
        assert!(reserved.0 > generation.generation_id.0);
        assert_eq!(source.1.load(Ordering::Relaxed), 0);
        let mut published = generation.as_ref().clone();
        published.generation_id = reserved;
        if admitted {
            let mut total = 0u64;
            let mut funding = execution.reserve_retained_memory(0).unwrap();
            let mut pay = |work: u64, bytes: u64| {
                crate::durability::artifact_payload_emission::request::account_work(
                    execution, work,
                )
                .unwrap();
                total = total.checked_add(bytes).unwrap();
                if let Some(funding) = &mut funding {
                    funding.resize(total).unwrap();
                }
                Ok::<_, ()>(())
            };
            let mut prepared = owner
                .prepare_generation_batch(vec![published.clone()], None, &mut pay)
                .unwrap();
            prepared.admit_actual_geometry(&mut pay).unwrap();
            prepared.publish_admitted();
        } else {
            owner.publish_generation(published.clone());
        }
        assert_eq!(owner.snapshot().generation_frontier(), reserved.0 + 1);
        assert_eq!(source.1.load(Ordering::Relaxed), 0);
        owner
            .definition_read_binding()
            .demand_checkpoint_image(execution)
            .unwrap();
        assert!(source.1.load(Ordering::Relaxed) > 0);
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        assert_eq!(
            runtime
                .index_access()
                .read_published_generation_for_observation(execution, index, &basis.observation())
                .unwrap()
                .unwrap()
                .generation_id,
            reserved
        );
        let loaded = owner.all_generations();
        assert_eq!(
            loaded
                .iter()
                .find(|g| g.generation_id == generation.generation_id)
                .map(Arc::as_ref),
            Some(generation.as_ref())
        );
        assert_eq!(
            loaded
                .iter()
                .find(|g| g.generation_id == reserved)
                .map(Arc::as_ref),
            Some(&published)
        );
    }
}
