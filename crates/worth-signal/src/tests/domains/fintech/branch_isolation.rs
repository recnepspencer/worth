use super::market_seed::MarketSeed;
use super::scales::FintechScale;
use super::scenarios::setup_seeded_world;
use super::truth_snapshot::FintechTruthSnapshot;
use crate::diagnostics::ReplayEventKind;
use crate::facade::*;

fn replay_is_branch_local(replay: &ReplaySlice, branch: &SignalBranchHandle) -> bool {
    replay
        .frames
        .iter()
        .all(|frame| frame.branch_id == branch.id)
}

#[test]
fn fintech_hostile_branch_replay_and_audit_workflow_stays_coherent() {
    let mut fixture = setup_seeded_world();
    fixture.assert_shape(FintechScale::smoke());

    let baseline = fixture.capture_active_checkpoint().unwrap();

    let analysis = fixture.open_branch("analysis-risk").unwrap();
    fixture.seed_market(MarketSeed::high_vol(17)).unwrap();
    let analysis_checkpoint = fixture.capture_active_checkpoint().unwrap();

    let analysis_replay_before = fixture.replay_for_branch(analysis.clone());
    fixture.inject_primary_market_rollback().unwrap();

    let analysis_replay_after = fixture.replay_for_branch(analysis.clone());
    assert!(
        analysis_replay_after.frames.len() > analysis_replay_before.frames.len(),
        "analysis branch should record the failed transaction"
    );
    assert!(
        analysis_replay_after
            .frames
            .iter()
            .any(|frame| frame.kind == ReplayEventKind::TransactionRolledBack),
        "analysis branch should retain rollback evidence"
    );

    fixture.restore_checkpoint(&analysis_checkpoint).unwrap();
    let restored_analysis = fixture.read_primary_audit_surface().unwrap();
    assert_eq!(restored_analysis, analysis_checkpoint.audit);

    let correction = fixture.open_branch("correction").unwrap();
    fixture.seed_market(MarketSeed::fx_dislocation(29)).unwrap();
    fixture.refresh_primary_audit_surface().unwrap();
    fixture.switch_branch(baseline.branch.clone()).unwrap();
    fixture.restore_checkpoint(&baseline).unwrap();
    let restored_main = fixture.read_primary_audit_surface().unwrap();
    assert_eq!(restored_main, baseline.audit);

    let main_replay = fixture.replay_for_branch(baseline.branch.clone());
    let correction_replay = fixture.replay_for_branch(correction.clone());
    assert!(
        correction_replay
            .frames
            .iter()
            .any(|frame| frame.kind == ReplayEventKind::BranchSwitched),
        "correction branch should preserve its branch activation event"
    );
    assert!(replay_is_branch_local(&main_replay, &baseline.branch));

    fixture.switch_branch(correction.clone()).unwrap();
    let correction_lineage = fixture.main_risk_lineage();
    assert!(correction_lineage.iter().any(|record| {
        matches!(
            record.kind,
            LineageRecordKind::SnapshotRestore { .. }
                | LineageRecordKind::ArtifactTransition {
                    transition: ArtifactTransitionKind::Replaced
                        | ArtifactTransitionKind::Refreshed { .. },
                    ..
                }
        )
    }));

    let around_snapshot = fixture.replay_around_saved_snapshot(&analysis_checkpoint.snapshot);
    assert!(around_snapshot
        .frames
        .iter()
        .any(|frame| frame.snapshot_id == Some(analysis_checkpoint.snapshot.meta.snapshot_id)));

    let current = fixture.current_branch();
    assert_eq!(current.name, "correction");
    assert_eq!(
        fixture.branch_head_snapshot_id(baseline.branch.clone()),
        Some(baseline.snapshot.meta.snapshot_id)
    );
}

#[test]
fn fintech_cross_branch_churn_keeps_branch_truth_from_leaking() {
    let mut fixture = setup_seeded_world();
    fixture.assert_shape(FintechScale::smoke());

    let main_checkpoint = fixture.capture_active_checkpoint().unwrap();

    let analysis = fixture.open_branch("analysis-isolated").unwrap();
    fixture.seed_market(MarketSeed::high_vol(17)).unwrap();
    fixture.bump_primary_market(7, 3, 1, 0).unwrap();
    let analysis_checkpoint = fixture.capture_active_checkpoint().unwrap();

    fixture
        .switch_branch(main_checkpoint.branch.clone())
        .unwrap();
    fixture.restore_checkpoint(&main_checkpoint).unwrap();
    let correction = fixture.open_branch("correction-isolated").unwrap();
    fixture.seed_market(MarketSeed::fx_dislocation(29)).unwrap();
    fixture.bump_primary_market(1, 6, 4, 3).unwrap();
    let correction_checkpoint = fixture.capture_active_checkpoint().unwrap();

    fixture.switch_branch(analysis.clone()).unwrap();
    fixture.restore_checkpoint(&analysis_checkpoint).unwrap();
    let analysis_audit = fixture.read_primary_audit_surface().unwrap();
    let analysis_market = fixture.read_primary_market_source().unwrap();

    fixture.switch_branch(correction.clone()).unwrap();
    fixture.restore_checkpoint(&correction_checkpoint).unwrap();
    let correction_audit = fixture.read_primary_audit_surface().unwrap();
    let correction_market = fixture.read_primary_market_source().unwrap();

    fixture
        .switch_branch(main_checkpoint.branch.clone())
        .unwrap();
    fixture.restore_checkpoint(&main_checkpoint).unwrap();
    let main_truth = FintechTruthSnapshot::capture(
        &mut fixture,
        &[
            ("main", main_checkpoint.branch.clone()),
            ("analysis", analysis.clone()),
            ("correction", correction.clone()),
        ],
        &[
            ("main", main_checkpoint.branch.clone()),
            ("analysis", analysis.clone()),
            ("correction", correction.clone()),
        ],
        1,
        1,
    )
    .unwrap();

    assert_eq!(main_truth.primary_audit, main_checkpoint.audit);
    assert_eq!(analysis_audit, analysis_checkpoint.audit);
    assert_eq!(correction_audit, correction_checkpoint.audit);
    assert_ne!(analysis_market, correction_market);
    assert_ne!(analysis_market, main_truth.primary_market);
    assert_ne!(correction_market, main_truth.primary_market);

    let main_replay = main_truth.replays.get("main").unwrap();
    let analysis_replay = main_truth.replays.get("analysis").unwrap();
    let correction_replay = main_truth.replays.get("correction").unwrap();
    assert!(replay_is_branch_local(main_replay, &main_checkpoint.branch));
    assert!(replay_is_branch_local(analysis_replay, &analysis));
    assert!(replay_is_branch_local(correction_replay, &correction));

    assert_eq!(
        main_truth.branch_heads.get("main"),
        Some(&Some(main_checkpoint.snapshot.meta.snapshot_id))
    );
    assert_eq!(
        main_truth.branch_heads.get("analysis"),
        Some(&Some(analysis_checkpoint.snapshot.meta.snapshot_id))
    );
    assert_eq!(
        main_truth.branch_heads.get("correction"),
        Some(&Some(correction_checkpoint.snapshot.meta.snapshot_id))
    );
}
