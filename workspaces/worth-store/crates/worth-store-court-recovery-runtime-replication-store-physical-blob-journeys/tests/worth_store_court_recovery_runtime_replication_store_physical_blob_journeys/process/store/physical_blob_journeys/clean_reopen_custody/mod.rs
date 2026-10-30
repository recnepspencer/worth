//! Ordinary reopen's own checkpoint custody on tier-anchored roots, and the
//! retained released-drop WAL member that still requires C.8.

#[allow(unused_imports)]
use crate::physical_blob_journeys::clean_reopen_custody::*;

use {
    crate::physical_blob_journeys::blob_crash::establish_recovery_frontier,
    crate::physical_blob_journeys::blob_crash::kill_at,
    crate::physical_blob_journeys::blob_reclaim_released_crash::TIER_ROLE,
    worth_store_testkit_store::fixture::assert_anchored_open_requires_recovered_custody,
    worth_store_testkit_store::fixture::assert_released_open_requires_recovered_custody,
    worth_store_testkit_store::fixture::configuration,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};

#[test]
fn tier_certified_checkpoint_reopens_ordinarily_and_checkpoints_again() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    serving
        .certification_activate_tier_epoch(placement())
        .unwrap();
    checkpoint(&serving, 0x61);
    serving.close();
    assert_eq!(
        certificate_kinds(directory.path()),
        [TIER_CERTIFICATE_KIND, RELEASE_CERTIFICATE_KIND]
    );

    let reopened = serving_from_open(directory.path());
    checkpoint(&reopened, 0x62);
    reopened.close();
    assert_eq!(
        certificate_kinds(directory.path()),
        [TIER_CERTIFICATE_KIND, RELEASE_CERTIFICATE_KIND],
        "the reopened tier custody re-certifies the anchor"
    );
}

#[test]
fn tier_activated_after_the_selected_checkpoint_still_requires_recovery() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    establish_recovery_frontier(&serving);
    serving
        .certification_activate_tier_epoch(placement())
        .unwrap();
    serving.close();
    assert_eq!(
        certificate_kinds(directory.path()),
        [RELEASE_CERTIFICATE_KIND]
    );
    assert_released_open_requires_recovered_custody(directory.path(), configuration().0);
}

#[test]
fn retained_released_drop_wal_member_keeps_ordinary_reopen_custody_unavailable() {
    let world = kill_at(
        "crash-released-first-wal",
        std::time::Duration::from_secs(240),
    );
    let reopened = serving_from_open(&world.root);
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x63; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    assert!(
        matches!(
            reopened.checkpoints().start(request).into_raw(),
            worth_proof::TransitionOutcome::Failed(worth_store::physical_runtime::PhysicalCheckpointStartFailure::Capture(
                worth_store::physical_runtime::PhysicalCheckpointCaptureFailureKind::CheckpointCustodyUnavailable
            ))
        ),
        "a retained released drop requires C8 custody"
    );
    reopened.close();
}

#[test]
fn tier_anchored_root_with_a_retained_released_drop_requires_recovery_before_serving() {
    let world = kill_at(TIER_ROLE, std::time::Duration::from_secs(240));
    assert_eq!(
        certificate_kinds(&world.root),
        [TIER_CERTIFICATE_KIND, RELEASE_CERTIFICATE_KIND]
    );
    assert_anchored_open_requires_recovered_custody(&world.root);
}
