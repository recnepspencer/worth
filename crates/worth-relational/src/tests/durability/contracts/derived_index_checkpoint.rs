use super::*;

fn name_index(runtime: &crate::runtime::RelationalRuntime) -> DerivedIndexDefinition {
    runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "checkpoint-index-tail".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    })
}

fn build(
    runtime: &crate::runtime::RelationalRuntime,
    index: DerivedIndexId,
    outcome: &crate::transactions::data::CommitResult,
) -> crate::indexes::data::DerivedIndexGenerationId {
    build_on_branch(runtime, index, outcome, BranchId("main".into()))
}

fn build_on_branch(
    runtime: &crate::runtime::RelationalRuntime,
    index: DerivedIndexId,
    outcome: &crate::transactions::data::CommitResult,
    branch_id: BranchId,
) -> crate::indexes::data::DerivedIndexGenerationId {
    let built = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: outcome.commit.commit_id,
            branch_id,
            index_ids: vec![index],
        });
    assert!(built.failed_indexes.is_empty());
    built.generations[0].generation_id
}

#[test]
fn reclamation_and_checkpoint_keep_sibling_head_generation() {
    let mut runtime = persisted_runtime_with_test_schema();
    let first = create_entity_outcome(&runtime, "sibling-index-first");
    let sibling = create_branch_from_main(&runtime, "index-sibling");
    let index = name_index(&runtime);
    let sibling_commit =
        create_entity_outcome_on_branch(&runtime, "sibling-index-own-head", sibling.clone());
    let sibling_generation =
        build_on_branch(&runtime, index.index_id, &sibling_commit, sibling.clone());
    let main = create_entity_outcome(&runtime, "main-index-new-head");
    let main_generation = build(&runtime, index.index_id, &main);
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &sibling_commit);
    release_test_commit_snapshot(&runtime, &main);
    runtime.run_branch_root_reclamation_pass();
    assert!(runtime.indexes.generation(sibling_generation).is_some());
    assert!(runtime.indexes.generation(main_generation).is_some());
    runtime.durability_authority().checkpoint().unwrap();
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered.durability_recovery().recover(plan).unwrap();
    assert!(recovered.indexes.generation(sibling_generation).is_some());
    assert!(recovered.indexes.generation(main_generation).is_some());
}

#[test]
fn repeated_edits_checkpoint_only_the_live_index_payload() {
    let runtime = persisted_runtime_with_test_schema();
    let first = create_entity_outcome(&runtime, "checkpoint-size-0");
    let index = name_index(&runtime);
    build(&runtime, index.index_id, &first);
    release_test_commit_snapshot(&runtime, &first);
    for ordinal in 1..24 {
        let outcome = create_entity_outcome(&runtime, &format!("checkpoint-size-{ordinal}"));
        build(&runtime, index.index_id, &outcome);
        release_test_commit_snapshot(&runtime, &outcome);
    }
    let legacy_all = crate::indexes::data::DerivedIndexArtifacts::new(
        runtime.index_access().generations_snapshot(),
    );
    let legacy_bytes = rmp_serde::to_vec_named(&legacy_all).unwrap().len();
    let checkpoint = runtime.durability_authority().checkpoint().unwrap();
    let retained_bytes =
        rmp_serde::to_vec_named(checkpoint.derived_index_checkpoint.as_ref().unwrap())
            .unwrap()
            .len();
    eprintln!(
        "index checkpoint sections: legacy_all={legacy_bytes} retained_delta={retained_bytes}"
    );
    assert!(retained_bytes * 4 < legacy_bytes);
    assert!(runtime.index_access().generations_snapshot().len() < 24);
}

#[test]
fn checkpoint_index_artifact_survives_tail_replay_without_envelope_caches() {
    let runtime = persisted_runtime_with_test_schema();
    let first = create_entity_outcome(&runtime, "checkpoint-index-first");
    let index = name_index(&runtime);
    let first_generation = build(&runtime, index.index_id, &first);
    let checkpoint = runtime.durability_authority().checkpoint().unwrap();
    assert!(checkpoint.derived_index_artifacts.is_empty());
    assert_eq!(checkpoint.derived_index_checkpoint_format, 1);
    let second = create_entity_outcome(&runtime, "checkpoint-index-tail");
    let unjournaled_cache = build(&runtime, index.index_id, &second);
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered.durability_recovery().recover(plan).unwrap();
    assert!(recovered.indexes.generation(first_generation).is_some());
    assert!(recovered.indexes.generation(unjournaled_cache).is_none());
    let rebuilt = build(&recovered, index.index_id, &second);
    assert!(recovered.indexes.generation(rebuilt).is_some());
}

#[test]
fn checkpoint_index_artifact_missing_payload_denies_before_recovery() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "checkpoint-missing-index-payload");
    runtime.durability_authority().checkpoint().unwrap();
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    plan.checkpoint.as_mut().unwrap().derived_index_checkpoint = None;
    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered.durability_recovery().recover(plan).unwrap_err();
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
}

#[test]
fn reclamation_keeps_a_pinned_old_reader_until_release() {
    let mut runtime = persisted_runtime_with_test_schema();
    let first = create_entity_outcome(&runtime, "pinned-index-first");
    let index = name_index(&runtime);
    let first_generation = build(&runtime, index.index_id, &first);
    let middle = create_entity_outcome(&runtime, "pinned-index-middle");
    let middle_generation = build(&runtime, index.index_id, &middle);
    let latest = create_entity_outcome(&runtime, "pinned-index-latest");
    let latest_generation = build(&runtime, index.index_id, &latest);
    release_test_commit_snapshot(&runtime, &middle);
    release_test_commit_snapshot(&runtime, &latest);
    runtime.run_branch_root_reclamation_pass();
    assert!(runtime.indexes.generation(first_generation).is_some());
    assert!(runtime.indexes.generation(latest_generation).is_some());
    assert!(runtime.indexes.generation(middle_generation).is_none());
    runtime.durability_authority().checkpoint().unwrap();
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered.durability_recovery().recover(plan).unwrap();
    assert!(recovered.indexes.generation(first_generation).is_none());
    assert!(recovered.indexes.generation(latest_generation).is_some());
    release_test_commit_snapshot(&runtime, &first);
    runtime.run_branch_root_reclamation_pass();
    assert!(runtime.indexes.generation(first_generation).is_none());
    assert!(runtime.indexes.generation(latest_generation).is_some());
}
