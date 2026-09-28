use crate::facade::history::BranchId;
use crate::tests::support::{changed_entities, field_key, read_entity_field};

mod persistence;

use super::actions::{
    build_branch_scoped_case_index, capture_world_snapshot, checkpoint_world,
    commit_case_trade_after_savepoint, compact_world_store, correct_seeded_trade_candidate,
    diverge_case_trade_on_branch, emit_trade_correction_audit_record, merge_branch_into_main,
    open_analysis_branch, open_audit_branch, recover_persisted_world, recover_runtime_from_plan,
    refresh_risk_views, register_case_book_index, release_snapshot_handle,
    repair_seeded_failed_settlement, shock_market_on_branch, stress_seeded_intraday_risk,
};
use super::comparisons::{compare_case_truth, compare_replay_probe};
use super::complexity::{
    assert_counter_at_most, contract_ids, measure_world_action, workflow_budgets, ComplexityBudget,
};
use super::failure_injection::{
    corrupt_latest_checkpoint_file, drop_latest_parent_envelope_for_replay,
    invalid_savepoint_rollback_code, replay_latest_commit_on_wrong_branch,
    rollback_seeded_trade_correction_after_savepoint,
};
use super::fixture::FintechWorld;
use super::invariants::{
    assert_correction_case_transition, assert_cross_context_relations, assert_fixture_shape,
    assert_intraday_risk_case_transition, assert_merge_metadata_preserved,
    assert_named_truth_world, assert_observability_overlap_stable,
    assert_observability_surfaces_agree, assert_partitioned_aspect_state,
    assert_recovery_matches_truth, assert_replay_targets_branch,
    assert_settlement_repair_case_transition, assert_snapshot_release_contract,
};
use super::naming::{
    artifact_alias, invariant_id, read_alias, replay_alias, scenario_name, workflow_name,
};
use super::probes::{
    capture_case_truth_probe, capture_observability_probe, capture_recovery_probe,
    capture_replay_probe, read_snapshot_probe, read_version_probe, ProbeStage,
};
use super::scales::FintechScale;
use super::scenarios::{
    setup_selected_world, setup_world, setup_world_for, setup_world_for_failed_settlement_repair,
    setup_world_for_historical_visibility, setup_world_for_intraday_risk,
    setup_world_for_late_trade_correction, setup_world_with, FintechScenario,
};

pub(super) fn setup_smoke_world() -> FintechWorld {
    let world = setup_world_for(FintechScenario::SmokeBook);
    assert_fixture_shape(&world, FintechScale::smoke());
    world
}

#[test]
fn fintech_fixture_builds_partitioned_truth_with_cross_context_relations() {
    let world = setup_world();
    let read = world.read_latest();

    assert_named_truth_world(&world);
    assert_partitioned_aspect_state(&read);
    assert_cross_context_relations(&read);
}

#[test]
fn fintech_world_setup_helpers_build_expected_scales() {
    let default_world = setup_world();
    let scaled_world = setup_world_with(FintechScale::smoke());
    let history_world = setup_world_for_historical_visibility();
    let intraday_world = setup_world_for_intraday_risk();
    let correction_world = setup_world_for_late_trade_correction();
    let settlement_world = setup_world_for_failed_settlement_repair();

    assert_fixture_shape(&default_world, FintechScale::smoke());
    assert_fixture_shape(&scaled_world, FintechScale::smoke());
    assert_named_truth_world(&history_world);
    assert_named_truth_world(&intraday_world);
    assert_named_truth_world(&correction_world);
    assert_named_truth_world(&settlement_world);
}

#[test]
fn fintech_scenario_selectors_expose_canonical_cases_and_expected_invariants() {
    let (_world, selection) = setup_selected_world(FintechScenario::LateTradeCorrection);

    assert!(matches!(
        selection.scenario,
        FintechScenario::LateTradeCorrection
    ));
    assert_eq!(
        selection.canonical_case,
        super::fixture::FintechCaseRole::LateTradeCorrection
    );
    assert_eq!(selection.scenario_key, "late-trade-correction");
    assert!(!selection.expected_invariants.is_empty());
    assert!(selection.expected_artifacts.contains(&"diagnostics"));
    assert_eq!(
        selection.expected_read_alias,
        "trade-correction.read.post-mutation"
    );
    assert_eq!(selection.probe_prefix, "trade-correction");
    assert!(!selection.persisted);
    assert_eq!(
        scenario_name("trade-correction", "baseline"),
        "fintech.trade-correction.baseline"
    );
    assert_eq!(
        workflow_name("trade-correction", "replay"),
        "fintech.trade-correction.replay"
    );
    assert_eq!(
        artifact_alias("trade-correction", "read", "post-mutation"),
        "trade-correction.read.post-mutation"
    );
    assert_eq!(
        read_alias("trade-correction", "post-mutation"),
        "trade-correction.read.post-mutation"
    );
    assert_eq!(
        replay_alias("trade-correction", "analysis"),
        "trade-correction.replay.analysis"
    );
    assert_eq!(
        invariant_id("trade-correction", "analysis_branch_local"),
        "trade-correction:analysis_branch_local"
    );
}

#[test]
fn fintech_snapshot_release_preserves_historical_visibility_and_invalidates_handle_reads() {
    let mut world = setup_world_for(FintechScenario::HistoricalVisibility);
    let analysis = open_analysis_branch(&mut world);
    let baseline_snapshot = capture_world_snapshot(&mut world);
    let baseline_probe = read_snapshot_probe(
        &world,
        super::fixture::FintechCaseRole::LateTradeCorrection,
        &baseline_snapshot,
        ProbeStage::Baseline,
    );

    let _correction = correct_seeded_trade_candidate(&mut world, analysis.clone());
    let _audit = emit_trade_correction_audit_record(&mut world, analysis);

    let historical_after_mutation = read_snapshot_probe(
        &world,
        super::fixture::FintechCaseRole::LateTradeCorrection,
        &baseline_snapshot,
        ProbeStage::PostMutation,
    );
    let historical_after_release = {
        let version_id = baseline_snapshot.version_id;
        assert!(release_snapshot_handle(&mut world, &baseline_snapshot));
        assert!(world
            .runtime
            .read_truth()
            .read_snapshot(&baseline_snapshot)
            .is_none());
        read_version_probe(
            &world,
            super::fixture::FintechCaseRole::LateTradeCorrection,
            version_id,
            ProbeStage::PostRecovery,
        )
    };

    assert_snapshot_release_contract(
        &baseline_probe,
        &historical_after_mutation,
        &historical_after_release,
    );
}

#[test]
fn fintech_world_exposes_named_domain_probes_for_correction_risk_and_settlement() {
    let world = setup_world();
    let snapshot = world.latest_snapshot();

    let correction = world.read_query(&snapshot, world.packet_for_correction_probe(&snapshot));
    let risk = world.read_query(&snapshot, world.packet_for_intraday_risk_probe(&snapshot));
    let settlement = world.read_query(
        &snapshot,
        world.packet_for_settlement_repair_probe(&snapshot),
    );

    assert_eq!(correction.entities.len(), 3);
    assert_eq!(risk.entities.len(), 4);
    assert_eq!(settlement.entities.len(), 4);
}

#[test]
fn fintech_analysis_workflow_preserves_branching_snapshots_and_trade_correction() {
    let mut world = setup_smoke_world();
    let analysis = open_analysis_branch(&mut world);
    let baseline_snapshot = world.runtime.visibility_authority().snapshot();
    let baseline_probe = read_snapshot_probe(
        &world,
        super::fixture::FintechCaseRole::LateTradeCorrection,
        &baseline_snapshot,
        ProbeStage::Baseline,
    );
    let baseline_observability = capture_observability_probe(&world, ProbeStage::Baseline);

    let _shock = shock_market_on_branch(&mut world, analysis.clone());
    let _correction = correct_seeded_trade_candidate(&mut world, analysis.clone());
    let _audit = emit_trade_correction_audit_record(&mut world, analysis.clone());
    let _risk = refresh_risk_views(&mut world, analysis.clone());

    let main_read = world
        .runtime
        .read_truth()
        .read_snapshot(&baseline_snapshot)
        .unwrap();
    let analysis_read = world.read_latest();
    assert_ne!(main_read.entities().len(), 0);
    assert!(analysis_read
        .entities()
        .iter()
        .any(|entity| read_entity_field(entity, field_key("corrected")) == Some("true".into())));
    assert_eq!(
        world
            .runtime
            .history()
            .branch_head(&BranchId("analysis".to_string())),
        world.runtime.history().latest_commit()
    );
    let post_probe = capture_case_truth_probe(
        &world,
        super::fixture::FintechCaseRole::LateTradeCorrection,
        ProbeStage::PostMutation,
    );
    let post_observability = capture_observability_probe(&world, ProbeStage::PostMutation);
    assert_correction_case_transition(&baseline_probe, &post_probe);
    assert_observability_overlap_stable(&baseline_observability, &post_observability);
}

#[test]
fn fintech_intraday_risk_workflow_exposes_open_breach_on_analysis_branch() {
    let mut world = setup_smoke_world();
    let analysis = open_analysis_branch(&mut world);
    let baseline_probe = capture_case_truth_probe(
        &world,
        super::fixture::FintechCaseRole::IntradayRisk,
        ProbeStage::Baseline,
    );

    let _stress = stress_seeded_intraday_risk(&mut world, analysis.clone());
    let post_probe = capture_case_truth_probe(
        &world,
        super::fixture::FintechCaseRole::IntradayRisk,
        ProbeStage::PostMutation,
    );
    let analysis_read = world.read_latest();
    let replay_probe = capture_replay_probe(&mut world, analysis.clone());
    let post_replay_probe = capture_case_truth_probe(
        &world,
        super::fixture::FintechCaseRole::IntradayRisk,
        ProbeStage::PostReplay,
    );

    assert!(analysis_read.entities().iter().any(|entity| {
        read_entity_field(entity, field_key("entity_type")) == Some("limit_breach".into())
            && read_entity_field(entity, field_key("status")) == Some("open".into())
    }));
    assert_eq!(
        world
            .runtime
            .history()
            .branch_head(&BranchId("analysis".to_string())),
        world.runtime.history().latest_commit()
    );
    assert_intraday_risk_case_transition(&baseline_probe, &post_probe);
    assert!(compare_case_truth(&post_probe, &post_replay_probe).is_empty());
    assert!(compare_replay_probe(&replay_probe, &replay_probe).is_empty());
    assert_replay_targets_branch(&replay_probe, &analysis);
}

#[test]
fn fintech_settlement_repair_workflow_exposes_repaired_settlement_on_analysis_branch() {
    let mut world = setup_smoke_world();
    let analysis = open_analysis_branch(&mut world);
    let baseline_probe = capture_case_truth_probe(
        &world,
        super::fixture::FintechCaseRole::FailedSettlementRepair,
        ProbeStage::Baseline,
    );

    let _repair = repair_seeded_failed_settlement(&mut world, analysis.clone());
    let post_probe = capture_case_truth_probe(
        &world,
        super::fixture::FintechCaseRole::FailedSettlementRepair,
        ProbeStage::PostMutation,
    );
    let analysis_read = world.read_latest();

    assert!(analysis_read.entities().iter().any(|entity| {
        read_entity_field(entity, field_key("entity_type")) == Some("settlement".into())
            && read_entity_field(entity, field_key("status")) == Some("repaired".into())
    }));
    assert_eq!(
        world
            .runtime
            .history()
            .branch_head(&BranchId("analysis".to_string())),
        world.runtime.history().latest_commit()
    );
    assert_settlement_repair_case_transition(&baseline_probe, &post_probe);
}
