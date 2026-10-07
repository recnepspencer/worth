use super::market_seed::MarketSeed;
use super::scales::FintechScale;
use super::scenarios::setup_seeded_world;
use super::truth_comparison::compare_exact;
use super::truth_snapshot::FintechTruthSnapshot;
use crate::facade::*;

fn replay_mentions_snapshot(replay: &ReplaySlice, snapshot_id: SignalSnapshotId) -> bool {
    replay
        .frames
        .iter()
        .any(|frame| frame.snapshot_id == Some(snapshot_id))
}

fn lineage_has_recovery_event(lineage: &[LineageRecord]) -> bool {
    lineage.iter().any(|record| {
        matches!(
            record.kind,
            LineageRecordKind::SnapshotRestore { .. }
                | LineageRecordKind::ArtifactTransition {
                    transition: ArtifactTransitionKind::Replaced
                        | ArtifactTransitionKind::Refreshed { .. },
                    ..
                }
        )
    })
}

#[test]
fn fintech_snapshot_restore_after_partial_audit_refresh_recovers_full_truth() {
    let mut fixture = setup_seeded_world();
    fixture.assert_shape(FintechScale::smoke());

    let analysis = fixture.open_branch("analysis-partial-refresh").unwrap();
    fixture.seed_market(MarketSeed::high_vol(17)).unwrap();
    let expected_truth = FintechTruthSnapshot::capture_core(&mut fixture).unwrap();
    let expected_audit = expected_truth.primary_audit.clone();
    let expected_market = expected_truth.primary_market;
    let snapshot = fixture.capture_branch_snapshot(analysis).unwrap();

    fixture.bump_primary_market(7, 4, 3, 2).unwrap();
    let skewed_market = fixture.read_primary_market_source().unwrap();
    assert_ne!(skewed_market, expected_market);
    fixture.read_top_desk().unwrap();

    fixture
        .restore_saved_snapshot(fixture.current_branch(), &snapshot)
        .unwrap();
    let restored_truth = FintechTruthSnapshot::capture_core(&mut fixture).unwrap();
    assert_eq!(restored_truth.primary_audit, expected_audit);
    assert!(compare_exact(&restored_truth, &expected_truth).is_empty());

    let around_snapshot = fixture.replay_around_saved_snapshot(&snapshot);
    assert!(replay_mentions_snapshot(
        &around_snapshot,
        snapshot.meta.snapshot_id
    ));
    assert!(lineage_has_recovery_event(&fixture.main_risk_lineage()));
}
