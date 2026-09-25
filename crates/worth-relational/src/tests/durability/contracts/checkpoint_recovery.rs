use super::*;
use worth_foundational::{FoundationalBranchReferenceObservation, FoundationalBranchTarget};

#[test]
fn durability_contract_empty_checkpoint_restores_a_live_retained_head() {
    let runtime = persisted_runtime_with_test_schema();
    runtime.durability_authority().checkpoint().unwrap();
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered.durability_recovery().recover(plan).unwrap();

    let identity = recovered.main_branch_identity();
    let (_, basis) = recovered
        .observe_branch(&identity)
        .expect("an empty recovered branch still has a complete owner root");
    assert_eq!(basis.observation().selected_root_identity(), 0);
    assert_eq!(recovered.retention_cost_counters().head_installs, 1);
}

#[test]
fn durability_contract_checkpoint_tail_recovery_preserves_post_checkpoint_commits() {
    let runtime = persisted_runtime_with_test_schema();
    let main = create_entity_outcome(&runtime, "main-a");
    let _checkpoint = runtime.durability_authority().checkpoint().unwrap();
    let later = create_entity_outcome(&runtime, "main-b");
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let checkpoint_images = plan.checkpoint.as_ref().unwrap().partition_images.as_ptr();
    let mut recovered = persisted_runtime_with_test_schema();
    let outcome = recovered.durability_recovery().recover(plan).unwrap();

    assert_eq!(outcome.recovered_commits, 2);
    assert_eq!(outcome.coverage.checkpoint_commits, 1);
    assert_eq!(outcome.coverage.replayed_tail_commits, 1);
    assert_eq!(outcome.latest_commit, Some(later.commit.clone()));
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_string())),
        Some(later.commit.clone())
    );
    assert_eq!(
        recovered
            .replay()
            .canonical_commit_envelope(main.commit.commit_id)
            .unwrap()
            .commit
            .commit_id,
        main.commit.commit_id
    );
    let retained = recovered.durability.latest_checkpoint().unwrap();
    assert_eq!(retained.envelopes.len(), 1);
    assert_eq!(retained.partition_images.as_ptr(), checkpoint_images);
}

#[test]
fn durability_tail_replay_releases_each_commit_snapshot_beyond_live_capacity() {
    let runtime = persisted_runtime_with_test_schema_profile(
        crate::facade::config::RelationalRuntimeProfile::AiWorkflow,
    );
    create_entity(&runtime, "replay-capacity-checkpoint");
    runtime.durability_authority().checkpoint().unwrap();
    for index in 0..70 {
        create_entity(&runtime, &format!("replay-capacity-tail-{index}"));
    }
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema_profile(
        crate::facade::config::RelationalRuntimeProfile::AiWorkflow,
    );

    let outcome = recovered.durability_recovery().recover(plan).unwrap();

    assert_eq!(outcome.recovered_commits, 71);
    assert_eq!(
        recovered
            .storage_access()
            .storage_stats()
            .published_snapshot_handle_count,
        0,
        "replay closeout must not consume the live published-snapshot budget"
    );
}

#[test]
fn durability_contract_recovery_fails_closed_without_exact_branch_cells() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "branch-cell-required");
    runtime.durability_authority().checkpoint().unwrap();
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    plan.checkpoint
        .as_mut()
        .expect("checkpoint is selected")
        .branch_cells
        .clear();

    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(plan)
        .expect_err("recovery must not synthesize branch cells from legacy heads");
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
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "branch-cell-artifact-required");
    runtime.durability_authority().checkpoint().unwrap();
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let checkpoint = plan.checkpoint.as_mut().expect("checkpoint is selected");
    let main_cell = checkpoint
        .branch_cells
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

    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(plan)
        .expect_err("recovery must reject a branch target with no immutable artifact");
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("references missing commit artifact"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
    assert!(recovered
        .history()
        .branch_head(&BranchId("main".to_owned()))
        .is_none());
}

#[test]
fn durability_contract_recovery_rejects_tail_checkpoint_drift_before_mutation() {
    let runtime = persisted_runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "checkpoint-basis");
    runtime.durability_authority().checkpoint().unwrap();
    let second = create_entity_outcome(&runtime, "tail-commit");
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let tail = plan
        .tail_log
        .iter_mut()
        .map(|commit| commit.envelope_mut_for_test())
        .find(|envelope| envelope.commit.commit_id == second.commit.commit_id)
        .expect("tail contains the post-checkpoint commit");
    let checkpoint = tail
        .branch_cell_checkpoint
        .as_mut()
        .expect("tail envelope carries its exact pre-commit branch cell");
    checkpoint.truth_version = crate::branch::RelationalBranchVersion::new(
        checkpoint.truth_version.as_u64().saturating_add(1),
    );

    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(plan)
        .expect_err("recovery must reject a mismatching existing branch checkpoint");
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

#[test]
fn durability_contract_checkpoint_recovers_index_metadata() {
    let runtime = persisted_runtime_with_test_schema();
    let commit = create_entity_outcome(&runtime, "indexed");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity-name".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    let build = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: commit.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        });
    runtime.durability_authority().checkpoint().unwrap();
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered.durability_recovery().recover(plan).unwrap();

    let index_access = recovered.index_access();
    let generation = index_access
        .latest_generation(index.index_id, &BranchId("main".to_string()))
        .unwrap();
    assert_eq!(generation.generation_id, build.generations[0].generation_id);
    assert_eq!(generation.source_commit_id, commit.commit.commit_id);
}

#[test]
fn durability_contract_checkpoint_recovers_lineage_authority() {
    let runtime = persisted_runtime_with_test_schema();
    let first = create_entity_outcome(&runtime, "first");
    let second = create_entity_outcome(&runtime, "second");
    let first_entity = changed_entities(&first)[0];
    let second_entity = changed_entities(&second)[0];
    let first_lineage = runtime
        .lineage_access()
        .for_record(first_entity)
        .unwrap()
        .lineage_id;
    let second_lineage = runtime
        .lineage_access()
        .for_record(second_entity)
        .unwrap()
        .lineage_id;
    let checkpoint = runtime.durability_authority().checkpoint().unwrap();
    assert_eq!(
        checkpoint
            .lineage
            .digest_basis()
            .published_lineage_commit_count,
        runtime
            .history()
            .commit_envelopes_snapshot()
            .iter()
            .filter(|envelope| envelope.has_lineage_authority())
            .count()
    );
    assert_eq!(
        checkpoint
            .lineage
            .digest_basis()
            .published_lineage_event_count,
        runtime
            .history()
            .commit_envelopes_snapshot()
            .iter()
            .map(|envelope| envelope.lineage_digest_basis().lineage_event_count())
            .sum::<usize>()
    );
    assert_eq!(
        checkpoint
            .lineage
            .digest_basis()
            .published_lineage_decision_count,
        runtime
            .history()
            .commit_envelopes_snapshot()
            .iter()
            .map(|envelope| envelope.lineage_digest_basis().lineage_decision_count())
            .sum::<usize>()
    );
    assert_eq!(
        checkpoint.lineage.counters().node_count,
        checkpoint.lineage.nodes().len()
    );
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered.durability_recovery().recover(plan).unwrap();
    let graph = recovered
        .lineage_access()
        .graph(crate::facade::lineage::LineageGraphRequest {
            branch_id: BranchId("main".to_string()),
            traversal_basis:
                crate::facade::lineage::LineageGraphTraversalBasis::FullBranchGraphMaterialization,
        });

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.events.len(), 2);
    assert!(graph
        .events
        .iter()
        .all(|event| event.kind == LineageEventKind::Create));
    assert_eq!(
        recovered
            .lineage_access()
            .for_record(first_entity)
            .unwrap()
            .lineage_id(),
        first_lineage
    );
    assert_eq!(
        recovered
            .lineage_access()
            .for_record(second_entity)
            .unwrap()
            .lineage_id(),
        second_lineage
    );
}

#[test]
fn durability_contract_corrupt_latest_checkpoint_falls_back_to_prior_valid_checkpoint() {
    let runtime = persisted_runtime_with_test_schema();
    let first = create_entity_outcome(&runtime, "first");
    runtime.durability_authority().checkpoint().unwrap();
    let second = create_entity_outcome(&runtime, "second");
    runtime.durability_authority().checkpoint().unwrap();
    let store = runtime
        .durability()
        .recovery_plan(
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .store
        .unwrap();
    let latest_checkpoint = store.checkpoints.last().unwrap();
    std::fs::write(&latest_checkpoint.path, b"{not-json").unwrap();

    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    let outcome = recovered.durability_recovery().recover(plan).unwrap();

    assert_eq!(outcome.latest_commit, Some(second.commit.clone()));
    assert!(!outcome
        .integrity_report
        .skipped_corrupt_checkpoints
        .is_empty());
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_string())),
        Some(second.commit.clone())
    );
    assert!(recovered
        .replay()
        .canonical_commit_envelope(first.commit.commit_id)
        .is_some());
}

#[test]
fn durability_contract_compaction_only_removes_segments_covered_by_checkpoint() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "a");
    create_entity_outcome(&runtime, "b");
    runtime.durability_authority().checkpoint().unwrap();
    create_entity_outcome(&runtime, "c");
    let before = runtime
        .durability()
        .recovery_plan(
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .store
        .unwrap();

    let compaction = runtime.durability_authority().compact_store().unwrap();
    let after = runtime
        .durability()
        .recovery_plan(
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .store
        .unwrap();

    assert!(!before.segments.is_empty());
    assert!(after.segments.len() <= before.segments.len());
    assert_eq!(after.segments.len(), compaction.retained_segments.len());
}
