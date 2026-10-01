//! Genuine Store activation and selected C8 checkpoint ancestry. Serving
//! remains sealed until Store's same-media tier rejoin is installed.

use super::*;
use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    checkpoint_certificate_frame_bytes, decode_checkpoint_certificate, CheckpointCertificateKind,
    CheckpointStreamFooter, ReleaseCheckpointCertificateV1, ReleaseCheckpointNoReleaseV1,
    CHECKPOINT_CERTIFICATE_PREFIX_BYTES, CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES,
};

#[test]
fn selected_tier_epoch_reopens_with_positive_no_release_checkpoint_ancestry() {
    let world = initialized_recovery_world("tier-positive-checkpoint-reopen");
    let root = world.retained_root().path().to_path_buf();
    let first = selected_no_release(&root);
    assert_eq!(first.prior_checkpoint_sequence(), 0);
    assert_eq!(first.prior_root_sha256(), [0; 32]);
    assert_eq!(first.prior_marker_payload_sha256(), [0; 32]);

    let epoch = world
        .serving()
        .certification_activate_tier_epoch(world.placement())
        .expect("typed one-time tier activation");
    assert!(epoch > 0);
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x6e; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("anchored checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    let successor = selected_no_release(&root);
    assert_eq!(
        successor.prior_checkpoint_sequence(),
        first.checkpoint().sequence().get()
    );
    assert_eq!(successor.prior_root_sha256(), first.root_sha256());
    let first_payload_digest: [u8; 32] = Sha256::digest(first.encode()).into();
    assert_eq!(
        successor.prior_marker_payload_sha256(),
        first_payload_digest
    );
    let retained = world.retained_root();
    drop(world);
    let output = run_entry(retained.path());
    assert!(
        output.status.success(),
        "selected C8 tier recovery failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn selected_tier_epoch_opens_serving_with_genuine_no_release_custody() {
    use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

    let world = initialized_recovery_world("tier-no-release-serving");
    world
        .serving()
        .certification_activate_tier_epoch(world.placement())
        .expect("typed one-time tier activation");
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x6f; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("anchored checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);

    let worker = std::thread::Builder::new()
        .name("certified-tier-recovery".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let outcome = WorthStoreRecovery::certification_recover_with_custody_pauses(
                super::certified_release_serving::request(&root),
                |_| {},
                || {},
            );
            let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("genuine selected tier custody failed Store rejoin: {outcome:?}")
            };
            let seal = handoff
                .into_core()
                .into_checkpoint_custody()
                .expect("selected tier custody seal");
            super::certified_release_serving::open_serving_with_seal(&root, seal);
        })
        .expect("tier recovery worker");
    worker.join().expect("tier recovery worker did not panic");
}

fn selected_no_release(root: &Path) -> ReleaseCheckpointNoReleaseV1 {
    let bytes =
        std::fs::read(root.join("families/checkpoint.current")).expect("selected checkpoint bytes");
    let footer_start = bytes.len() - CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES;
    let footer = CheckpointStreamFooter::decode_record(&bytes[footer_start..])
        .expect("selected certified footer");
    let mut offset = footer_start - footer.certificate_record_bytes() as usize;
    let mut marker = None;
    for _ in 0..footer.certificate_record_count() {
        let length = checkpoint_certificate_frame_bytes(
            &bytes[offset..offset + CHECKPOINT_CERTIFICATE_PREFIX_BYTES],
        )
        .expect("selected certificate length");
        let (kind, payload) = decode_checkpoint_certificate(&bytes[offset..offset + length])
            .expect("selected certificate frame");
        if kind == CheckpointCertificateKind::ReleasedDrop {
            let ReleaseCheckpointCertificateV1::NoRelease(value) =
                ReleaseCheckpointCertificateV1::decode(payload).expect("typed NoRelease marker")
            else {
                panic!("tier-only checkpoint must not carry release custody")
            };
            assert!(marker.replace(value).is_none());
        }
        offset += length;
    }
    assert_eq!(offset, footer_start);
    marker.expect("positive NoRelease selected marker")
}
