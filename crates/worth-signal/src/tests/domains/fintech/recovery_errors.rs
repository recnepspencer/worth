use super::scales::FintechScale;
use super::scenarios::setup_seeded_world;
use super::truth_comparison::compare_exact;
use super::truth_snapshot::FintechTruthSnapshot;

#[test]
fn fintech_cross_branch_restore_rejection_preserves_branch_heads_and_truth() {
    let mut world = setup_seeded_world();
    world.assert_shape(FintechScale::smoke());

    let main = world.current_branch();
    let expected_truth = FintechTruthSnapshot::capture_core(&mut world).unwrap();
    let main_snapshot = world.capture_world_snapshot();
    let feature = world.open_branch("feature-invalid-restore").unwrap();

    let err = world.attempt_cross_branch_restore(feature.clone(), &main_snapshot);
    assert!(err.is_err(), "cross-branch restore should be rejected");

    world.switch_branch(main.clone()).unwrap();
    let truth_after_rejection = FintechTruthSnapshot::capture_core(&mut world).unwrap();
    assert!(compare_exact(&truth_after_rejection, &expected_truth).is_empty());

    let feature_snapshot = world.capture_branch_snapshot(feature.clone()).unwrap();
    assert_eq!(feature_snapshot.meta.branch_id, feature.id);
    assert_eq!(
        world.branch_head_snapshot_id(feature),
        Some(feature_snapshot.meta.snapshot_id)
    );
    assert_eq!(
        world.branch_head_snapshot_id(main.clone()),
        Some(main_snapshot.meta.snapshot_id)
    );
}

#[test]
fn fintech_incompatible_profile_restore_rejection_preserves_snapshot_contract() {
    let mut world = setup_seeded_world();
    world.assert_shape(FintechScale::smoke());

    let analysis = world.open_branch("analysis-incompatible-profile").unwrap();
    let snapshot = world.capture_branch_snapshot(analysis.clone()).unwrap();
    let original_profile = snapshot.meta.core_storage_profile.clone();

    let err = world.attempt_incompatible_profile_restore(analysis.clone(), &snapshot);
    assert!(
        err.is_err(),
        "restore should reject snapshots with incompatible storage profiles"
    );

    let restored = world.capture_branch_snapshot(analysis).unwrap();
    assert_eq!(restored.meta.core_storage_profile, original_profile);
}
