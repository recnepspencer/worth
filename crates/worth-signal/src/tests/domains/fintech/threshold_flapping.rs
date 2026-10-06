use super::scales::FintechScale;
use super::scenarios::setup_world;
use crate::diagnostics::ReplayEventKind;
use crate::facade::*;

fn rollback_count(replay: &ReplaySlice) -> usize {
    replay
        .frames
        .iter()
        .filter(|frame| frame.kind == ReplayEventKind::TransactionRolledBack)
        .count()
}

fn replay_mentions_snapshot(replay: &ReplaySlice, snapshot_id: SignalSnapshotId) -> bool {
    replay
        .frames
        .iter()
        .any(|frame| frame.snapshot_id == Some(snapshot_id))
}

#[test]
fn fintech_threshold_flap_rollback_storm_preserves_condition_and_restore_coherence() {
    let mut fixture = setup_world();
    fixture.assert_shape(FintechScale::smoke());

    let analysis = fixture.open_branch("analysis-threshold").unwrap();
    let checkpoint = fixture.capture_active_checkpoint().unwrap();
    let baseline_threshold = fixture.read_primary_threshold().unwrap();

    fixture.bump_primary_market(1, 0, 0, 0).unwrap();
    let threshold_after_small = fixture.read_primary_threshold().unwrap();
    assert_eq!(threshold_after_small, baseline_threshold);

    fixture.bump_primary_market(3, 0, 0, 0).unwrap();
    let threshold_after_large = fixture.read_primary_threshold().unwrap();
    assert_ne!(threshold_after_large, baseline_threshold);

    for _ in 0..3 {
        fixture.inject_primary_market_rollback().unwrap();
    }

    let replay_after_storm = fixture.replay_for_branch(analysis);
    assert!(rollback_count(&replay_after_storm) >= 3);

    fixture.restore_checkpoint(&checkpoint).unwrap();
    let restored_threshold = fixture.read_primary_threshold().unwrap();
    let restored_audit = fixture.read_primary_audit_surface().unwrap();
    assert_eq!(restored_threshold, baseline_threshold);
    assert_eq!(restored_audit, checkpoint.audit);

    let around_checkpoint = fixture.replay_around_saved_snapshot(&checkpoint.snapshot);
    assert!(replay_mentions_snapshot(
        &around_checkpoint,
        checkpoint.snapshot.meta.snapshot_id
    ));
}
