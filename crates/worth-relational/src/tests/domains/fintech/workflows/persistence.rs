use super::*;
use crate::facade::query::PlannedQueryPacket;
use crate::tests::domains::fintech::fixture::FintechCaseRole;
use crate::tests::support::{field_key, read_entity_field};

#[test]
fn fintech_persisted_workflow_recovers_checkpoint_tail_and_keeps_queryable_portfolio_state() {
    let mut world = setup_world_for(FintechScenario::PersistedSmokeBook);
    let analysis = open_analysis_branch(&mut world);
    let _shock = shock_market_on_branch(&mut world, analysis.clone());
    let _checkpoint = checkpoint_world(&mut world).unwrap();
    let _correction = correct_seeded_trade_candidate(&mut world, analysis);
    let expected = {
        let plan = world.runtime.durability().recovery_plan(
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        );
        let mut recovered = FintechWorld::setup_persisted_world().runtime;
        let outcome = recovered.durability_recovery().recover(plan).unwrap();
        capture_recovery_probe(&recovered, &outcome)
    };

    let (recovered, outcome) = recover_persisted_world(&world).unwrap();
    let recovered_snapshot = recovered.visibility_authority().snapshot();
    let packet = {
        let context = recovered
            .read_truth()
            .query_plan_context(&recovered_snapshot)
            .expect("recovered portfolio query plan context");
        PlannedQueryPacket::explicit_targets(
            "portfolio-check",
            context,
            world
                .packet_for_portfolio_probe(&world.latest_snapshot())
                .explicit_target_refs()
                .expect("portfolio probe uses explicit targets")
                .to_vec(),
        )
    };
    let result = recovered
        .read_truth()
        .execute_query_plan(
            recovered
                .read_truth()
                .plan_query_packet(&recovered_snapshot, packet)
                .expect("planned recovered query"),
        )
        .expect("executed recovered query")
        .result;
    let before_probe = capture_case_truth_probe(
        &world,
        FintechCaseRole::BaselinePortfolio,
        ProbeStage::PostMutation,
    );
    let recovery_probe = capture_recovery_probe(&recovered, &outcome);
    recovered
        .snapshots()
        .release_snapshot(&recovered_snapshot)
        .unwrap();

    assert_eq!(result.entities.len(), 3);
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("analysis".to_string())),
        recovered.history().latest_commit()
    );
    let after_probe = read_snapshot_probe(
        &world,
        FintechCaseRole::BaselinePortfolio,
        &world.latest_snapshot(),
        ProbeStage::PostRecovery,
    );
    assert!(compare_case_truth(&before_probe, &after_probe).is_empty());
    assert_recovery_matches_truth(&expected, &recovery_probe);
}

#[test]
fn fintech_branch_divergence_merge_and_savepoint_verbs_stay_case_local() {
    let mut world = setup_smoke_world();
    let analysis = open_analysis_branch(&mut world);
    let audit = open_audit_branch(&mut world);

    let rollback = rollback_seeded_trade_correction_after_savepoint(&mut world, analysis.clone());
    assert!(rollback.has_effects());
    assert!(rollback.summary().has_restored_entity());

    let _saved = commit_case_trade_after_savepoint(&mut world, analysis.clone());
    let _diverged = diverge_case_trade_on_branch(
        &mut world,
        audit.clone(),
        FintechCaseRole::LateTradeCorrection,
        1_610_000,
    );
    let merged = merge_branch_into_main(&mut world, audit.clone());
    let (merge_parent_branches, merge_base_count, parent_count) = {
        let replay = world.runtime.replay();
        let envelope = replay
            .canonical_commit_envelope(merged.commit.commit_id)
            .expect("merged commit should have canonical envelope");
        (
            envelope.merge_parent_branches.clone(),
            envelope.merge_base_commits.len(),
            envelope.commit.parents.len(),
        )
    };

    assert_eq!(merged.commit.branch_id, BranchId("main".to_string()));
    assert_eq!(
        world
            .runtime
            .history()
            .branch_head(&audit)
            .map(|commit| commit.commit_id),
        Some(merged.commit.parents[1])
    );
    assert_ne!(
        world
            .runtime
            .history()
            .branch_head(&BranchId("analysis".to_string()))
            .map(|commit| commit.commit_id),
        Some(merged.commit.commit_id)
    );
    assert_merge_metadata_preserved(
        &merge_parent_branches,
        merge_base_count,
        parent_count,
        &audit,
        2,
    );
}

#[test]
fn fintech_failure_injection_helpers_cover_savepoints_replay_and_checkpoint_corruption() {
    let mut world = setup_smoke_world();
    let analysis = open_analysis_branch(&mut world);
    let rollback = rollback_seeded_trade_correction_after_savepoint(&mut world, analysis.clone());
    let invalid_code = invalid_savepoint_rollback_code(&mut world, analysis.clone());

    assert!(rollback.has_effects());
    assert!(rollback.summary().has_restored_entity());
    assert_eq!(invalid_code, DiagnosticCode::InvalidSavepoint);

    let _correction = correct_seeded_trade_candidate(&mut world, analysis);
    let removed = drop_latest_parent_envelope_for_replay(&world.runtime);
    assert!(removed.is_some());
    let wrong_branch = replay_latest_commit_on_wrong_branch(&world.runtime).unwrap();
    assert_eq!(
        wrong_branch.failure,
        Some(ReplayFailureClass::BranchMismatch)
    );

    let mut persisted = setup_world_for(FintechScenario::PersistedSmokeBook);
    checkpoint_world(&mut persisted).unwrap();
    let path = corrupt_latest_checkpoint_file(&persisted.runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    ));
    assert!(path.is_some());
}

#[test]
fn fintech_complexity_hooks_measure_seeded_workflows() {
    let mut world = setup_smoke_world();
    let analysis = open_analysis_branch(&mut world);
    let contracts = contract_ids(&world);
    let clone_budget = workflow_budgets()
        .into_iter()
        .find(|budget| budget.label == "full_state_clones")
        .unwrap_or(ComplexityBudget {
            label: "full_state_clones",
            max: 0,
            selector: |counters| counters.full_state_clones,
        });

    assert!(contracts.contains(&"runtime.partition_local_commit"));

    let counters = measure_world_action(&mut world, |world| {
        let _ = correct_seeded_trade_candidate(world, analysis.clone());
    });
    assert_counter_at_most(
        &counters,
        clone_budget.selector,
        clone_budget.max,
        clone_budget.label,
    );
}

#[test]
fn fintech_persisted_workflow_supports_compaction_after_checkpoint() {
    let mut world = setup_world_for(FintechScenario::PersistedSmokeBook);
    let analysis = open_analysis_branch(&mut world);
    let _shock = shock_market_on_branch(&mut world, analysis);
    checkpoint_world(&mut world).unwrap();
    let pre_compaction_probe = capture_case_truth_probe(
        &world,
        FintechCaseRole::BaselinePortfolio,
        ProbeStage::PostMutation,
    );

    let compaction = compact_world_store(&mut world).unwrap();
    let (recovered, outcome) = recover_persisted_world(&world).unwrap();
    let recovered_probe = capture_recovery_probe(&recovered, &outcome);

    assert!(
        !compaction.removed_segments.is_empty() || !compaction.retained_segments.is_empty(),
        "compaction should report at least one segment decision"
    );
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("analysis".to_string())),
        recovered.history().latest_commit()
    );
    let recovered_snapshot = recovered.visibility_authority().snapshot();
    let packet = {
        let context = recovered
            .read_truth()
            .query_plan_context(&recovered_snapshot)
            .expect("recovered portfolio query plan context");
        PlannedQueryPacket::explicit_targets(
            "portfolio-check",
            context,
            world
                .packet_for_portfolio_probe(&world.latest_snapshot())
                .explicit_target_refs()
                .expect("portfolio probe uses explicit targets")
                .to_vec(),
        )
    };
    let result = recovered
        .read_truth()
        .execute_query_plan(
            recovered
                .read_truth()
                .plan_query_packet(&recovered_snapshot, packet)
                .expect("planned recovered portfolio query"),
        )
        .expect("executed recovered portfolio query")
        .result;
    assert_eq!(result.entities.len(), pre_compaction_probe.entity_count);
    assert!(recovered_probe.latest_commit_id.is_some());
}

#[test]
fn fintech_index_and_lineage_authority_survive_hostile_workflow_recovery() {
    let mut world = setup_world_for(FintechScenario::PersistedSmokeBook);
    let analysis = open_analysis_branch(&mut world);
    let correction = correct_seeded_trade_candidate(&mut world, analysis.clone());
    let corrected_trade = changed_entities(&correction)[0];
    let corrected_lineage = world
        .runtime
        .lineage_access()
        .for_record(corrected_trade)
        .expect("corrected trade lineage")
        .lineage_id();
    let _audit = emit_trade_correction_audit_record(&mut world, analysis.clone());
    let index = register_case_book_index(&mut world);
    let build = build_branch_scoped_case_index(
        &mut world,
        index.index_id,
        analysis,
        correction.commit.clone(),
    );
    checkpoint_world(&mut world).unwrap();

    let (recovered, _outcome) = recover_persisted_world(&world).unwrap();
    let index_access = recovered.index_access();
    let generation = index_access
        .latest_generation(index.index_id, &BranchId("analysis".to_string()))
        .expect("analysis index generation should recover");

    assert!(build.failed_indexes.is_empty());
    assert_eq!(generation.generation_id, build.generations[0].generation_id);
    assert_eq!(generation.source_commit_id, correction.commit.commit_id);
    assert_eq!(
        recovered
            .lineage_access()
            .for_record(corrected_trade)
            .expect("recovered corrected trade lineage")
            .lineage_id(),
        corrected_lineage
    );
    assert_eq!(
        generation.source_branch_id,
        BranchId("analysis".to_string())
    );
}

#[test]
fn fintech_observability_surfaces_agree_for_hostile_correction_workflow() {
    let mut world = setup_smoke_world();
    let analysis = open_analysis_branch(&mut world);

    let _shock = shock_market_on_branch(&mut world, analysis.clone());
    let _correction = correct_seeded_trade_candidate(&mut world, analysis.clone());
    let _audit = emit_trade_correction_audit_record(&mut world, analysis.clone());
    let _risk = refresh_risk_views(&mut world, analysis);

    assert_observability_surfaces_agree(&world);
}

#[test]
fn fintech_recovery_falls_back_from_corrupt_latest_checkpoint_and_keeps_truth() {
    let mut world = setup_world_for(FintechScenario::PersistedSmokeBook);
    let analysis = open_analysis_branch(&mut world);
    let _shock = shock_market_on_branch(&mut world, analysis.clone());
    checkpoint_world(&mut world).unwrap();
    let correction = correct_seeded_trade_candidate(&mut world, analysis);
    checkpoint_world(&mut world).unwrap();
    let plan = world.runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let corrupted = corrupt_latest_checkpoint_file(&plan);
    assert!(corrupted.is_some());

    let (recovered, outcome) = recover_runtime_from_plan(world.runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    ))
    .unwrap();
    let recovered_snapshot = recovered.visibility_authority().snapshot();
    let packet = {
        let context = recovered
            .read_truth()
            .query_plan_context(&recovered_snapshot)
            .expect("recovered correction query plan context");
        PlannedQueryPacket::explicit_targets(
            "correction-probe",
            context,
            world
                .packet_for_correction_probe(&world.latest_snapshot())
                .explicit_target_refs()
                .expect("correction probe uses explicit targets")
                .to_vec(),
        )
    };
    let result = recovered
        .read_truth()
        .execute_query_plan(
            recovered
                .read_truth()
                .plan_query_packet(&recovered_snapshot, packet)
                .expect("planned recovered correction query"),
        )
        .expect("executed recovered correction query")
        .result;

    assert_eq!(outcome.latest_commit, Some(correction.commit.clone()));
    assert!(!outcome
        .integrity_report
        .skipped_corrupt_checkpoints
        .is_empty());
    assert!(result
        .entities
        .iter()
        .any(|entity| read_entity_field(entity, field_key("corrected")) == Some("true".into())));
}
use crate::facade::diagnostics::DiagnosticCode;
use crate::facade::replay::ReplayFailureClass;
