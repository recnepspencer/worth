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

fn scoped_name_index(runtime: &crate::runtime::RelationalRuntime) -> DerivedIndexDefinition {
    runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "scoped-checkpoint-index".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: true,
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
    runtime.run_index_generation_reclamation_pass();
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
fn live_fork_scoped_generation_readmits_from_shared_basis() {
    let runtime = persisted_runtime_with_test_schema();
    let anchor = create_entity_outcome(&runtime, "scoped-fork-checkpoint-anchor");
    let branch = create_branch_from_main(&runtime, "scoped-fork-checkpoint");
    let identity = runtime.branch_identity(&branch).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let index = scoped_name_index(&runtime);
    let built = runtime.index_authority().build_for_basis(
        DerivedIndexBuildRequest {
            source_commit_id: anchor.commit.commit_id,
            branch_id: branch.clone(),
            index_ids: vec![index.index_id],
        },
        &basis,
    );
    assert!(built.failed_indexes.is_empty());
    let generation = built.generations[0].generation_id;
    drop(basis);
    release_test_commit_snapshot(&runtime, &anchor);
    runtime.durability_authority().checkpoint().unwrap();
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered.durability_recovery().recover(plan).unwrap();
    assert!(recovered.indexes.generation(generation).is_some());
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
    assert_eq!(checkpoint.derived_index_checkpoint_format, 2);
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
    let mut old_format = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    old_format
        .checkpoint
        .as_mut()
        .unwrap()
        .derived_index_checkpoint_format = 1;
    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(old_format)
        .unwrap_err();
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
}

#[test]
fn checkpoint_recovery_denies_a_digest_valid_foreign_generation_affinity() {
    let runtime = persisted_runtime_with_test_schema();
    let commit = create_entity_outcome(&runtime, "foreign-affinity-index");
    let index = name_index(&runtime);
    build(&runtime, index.index_id, &commit);
    release_test_commit_snapshot(&runtime, &commit);
    runtime.durability_authority().checkpoint().unwrap();
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let checkpoint = plan.checkpoint.as_mut().unwrap();
    let mut generation = checkpoint
        .derived_index_checkpoint
        .as_ref()
        .unwrap()
        .readmit()
        .unwrap()
        .pop()
        .unwrap();
    generation.applicability.branch_id = BranchId("foreign".into());
    checkpoint.derived_index_checkpoint = Some(
        crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::capture(vec![
            std::sync::Arc::new(generation),
        ])
        .unwrap(),
    );
    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered.durability_recovery().recover(plan).unwrap_err();
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("branch affinity"));
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
    assert!(runtime.indexes.generation(middle_generation).is_some());
    runtime.run_index_generation_reclamation_pass();
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
    runtime.run_index_generation_reclamation_pass();
    assert!(runtime.indexes.generation(first_generation).is_none());
    assert!(runtime.indexes.generation(latest_generation).is_some());
}

#[test]
fn repeated_closed_branches_leave_no_latest_generation_residue() {
    let runtime = persisted_runtime_with_test_schema();
    let anchor = create_entity_outcome(&runtime, "closed-index-anchor");
    let index = scoped_name_index(&runtime);
    release_test_commit_snapshot(&runtime, &anchor);
    let mut runtime = runtime;
    for ordinal in 0..12 {
        let branch = create_branch_from_main(&runtime, &format!("closed-index-{ordinal}"));
        let identity = runtime.branch_identity(&branch).unwrap();
        let generation = {
            let (_, basis) = runtime.observe_branch(&identity).unwrap();
            let built = runtime.index_authority().build_for_basis(
                DerivedIndexBuildRequest {
                    source_commit_id: anchor.commit.commit_id,
                    branch_id: branch.clone(),
                    index_ids: vec![index.index_id],
                },
                &basis,
            );
            assert!(built.failed_indexes.is_empty());
            built.generations[0].generation_id
        };
        assert!(runtime
            .delete_branch(&identity)
            .unwrap()
            .deleted()
            .is_some());
        assert!(runtime.indexes.generation(generation).is_some());
        assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(1));
        assert!(runtime.indexes.generation(generation).is_none());
        assert!(runtime
            .index_access()
            .latest_generation(index.index_id, &branch)
            .is_none());
        assert_eq!(runtime.history.retired_branch_binding_count(), 0);
    }
    assert!(runtime.index_access().generations_snapshot().is_empty());
}

#[test]
fn deleted_branch_reader_retains_only_its_exact_scoped_generation() {
    let runtime = persisted_runtime_with_test_schema();
    let anchor = create_entity_outcome(&runtime, "pinned-deleted-fork-anchor");
    let index = scoped_name_index(&runtime);
    release_test_commit_snapshot(&runtime, &anchor);
    let mut runtime = runtime;
    let branch = create_branch_from_main(&runtime, "pinned-deleted-fork");
    let identity = runtime.branch_identity(&branch).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let built = runtime.index_authority().build_for_basis(
        DerivedIndexBuildRequest {
            source_commit_id: anchor.commit.commit_id,
            branch_id: branch.clone(),
            index_ids: vec![index.index_id],
        },
        &basis,
    );
    assert!(built.failed_indexes.is_empty());
    let generation = built.generations[0].generation_id;
    assert!(runtime
        .delete_branch(&identity)
        .unwrap()
        .deleted()
        .is_some());
    assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(0));
    assert!(runtime.indexes.generation(generation).is_some());
    runtime.durability_authority().checkpoint().unwrap();
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered.durability_recovery().recover(plan).unwrap();
    assert!(recovered.indexes.generation(generation).is_none());
    drop(basis);
    assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(1));
    assert!(runtime.indexes.generation(generation).is_none());
}

#[test]
fn historical_reader_from_live_head_survives_head_retirement() {
    let runtime = persisted_runtime_with_test_schema();
    let anchor = create_entity_outcome(&runtime, "historical-reader-anchor");
    let branch = create_branch_from_main(&runtime, "historical-reader-fork");
    let first =
        create_entity_outcome_on_branch(&runtime, "historical-reader-first", branch.clone());
    let index = scoped_name_index(&runtime);
    let first_generation = build_on_branch(&runtime, index.index_id, &first, branch.clone());
    release_test_commit_snapshot(&runtime, &anchor);
    release_test_commit_snapshot(&runtime, &first);
    let advanced_main = create_entity_outcome(&runtime, "historical-reader-main-advanced");
    release_test_commit_snapshot(&runtime, &advanced_main);
    let retained = crate::visibility::snapshot_states::HistoricalVisibilityBasis::resolve(
        &runtime,
        first.commit.version_id,
    )
    .unwrap();
    assert_eq!(retained.branch_id(), &branch);
    let next = create_entity_outcome_on_branch(&runtime, "historical-reader-next", branch.clone());
    let next_generation = build_on_branch(&runtime, index.index_id, &next, branch);
    release_test_commit_snapshot(&runtime, &next);
    assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(0));
    assert!(runtime.indexes.generation(first_generation).is_some());
    assert!(runtime.indexes.generation(next_generation).is_some());
    drop(retained);
    assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(1));
    assert!(runtime.indexes.generation(first_generation).is_none());
    assert!(runtime.indexes.generation(next_generation).is_some());
}
