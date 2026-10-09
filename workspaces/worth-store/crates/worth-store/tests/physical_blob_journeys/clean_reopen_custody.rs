//! Ordinary reopen's own checkpoint custody on tier-anchored roots, and the
//! retained released-drop WAL member that still requires C.8.

use std::{fs, path::Path, time::Duration};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalCheckpointCaptureFailureKind, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
    PhysicalCheckpointStartFailure, ServingPhysicalRuntime,
};

use super::{
    blob_crash::{establish_recovery_frontier, kill_at},
    blob_reclaim_released_crash::TIER_ROLE,
    fixture::{
        assert_anchored_open_requires_recovered_custody,
        assert_released_open_requires_recovered_custody, configuration, placement,
        serving_from_initialization, serving_from_open,
    },
};

const TIER_CERTIFICATE_KIND: u8 = 6;
const RELEASE_CERTIFICATE_KIND: u8 = 7;

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
fn tier_activated_after_the_selected_checkpoint_still_requires_c8() {
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
    let world = kill_at("crash-released-first-wal", Duration::from_secs(240));
    let reopened = serving_from_open(&world.root);
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x63; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    assert!(
        matches!(
            reopened.checkpoints().start(request).into_raw(),
            TransitionOutcome::Failed(PhysicalCheckpointStartFailure::Capture(
                PhysicalCheckpointCaptureFailureKind::CheckpointCustodyUnavailable
            ))
        ),
        "a retained released drop requires C8 custody"
    );
    reopened.close();
}

#[test]
fn tier_anchored_root_with_a_retained_released_drop_requires_c8_before_serving() {
    let world = kill_at(TIER_ROLE, Duration::from_secs(240));
    assert_eq!(
        certificate_kinds(&world.root),
        [TIER_CERTIFICATE_KIND, RELEASE_CERTIFICATE_KIND]
    );
    assert_anchored_open_requires_recovered_custody(&world.root);
}

/// Activates the tier epoch and certifies it in a selected checkpoint.
pub(super) fn certify_tier_epoch(serving: &ServingPhysicalRuntime) {
    serving
        .certification_activate_tier_epoch(placement())
        .unwrap();
    checkpoint(serving, 0x64);
}

fn checkpoint(serving: &ServingPhysicalRuntime, key: u8) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
}

/// Kinds of the selected checkpoint's certificate records, in stream order.
fn certificate_kinds(root: &Path) -> Vec<u8> {
    let bytes = fs::read(root.join("families/checkpoint.current")).unwrap();
    let mut kinds = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let payload = u32::from_le_bytes(bytes[offset + 12..offset + 16].try_into().unwrap());
        let kind = bytes[offset + 9];
        if matches!(kind, TIER_CERTIFICATE_KIND | RELEASE_CERTIFICATE_KIND) {
            kinds.push(kind);
        }
        offset += 16 + payload as usize + 4;
    }
    kinds
}
