//! Genuine Store activation and selected C8 checkpoint ancestry. Serving
//! remains sealed until Store's same-media tier rejoin is installed.

#[allow(unused_imports)]
use crate::production_entry::tier_epoch_reopen::*;
use sha2::Digest;
use {
    worth_store_testkit_recovery_runtime::production_entry::initialized_recovery_world,
    worth_store_testkit_recovery_runtime::production_entry::run_entry,
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
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x6e; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("anchored checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    let successor = selected_no_release(&root);
    assert_eq!(
        successor.prior_checkpoint_sequence(),
        first.checkpoint().sequence().get()
    );
    assert_eq!(successor.prior_root_sha256(), first.root_sha256());
    let first_payload_digest: [u8; 32] = sha2::Sha256::digest(first.encode()).into();
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
    use {
        worth_store_recovery_runtime::PhysicalRecoveryOutcome,
        worth_store_recovery_runtime::WorthStoreRecovery,
    };

    let world = initialized_recovery_world("tier-no-release-serving");
    world
        .serving()
        .certification_activate_tier_epoch(world.placement())
        .expect("typed one-time tier activation");
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x6f; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("anchored checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);

    let worker = std::thread::Builder::new()
        .name("certified-tier-recovery".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let outcome = WorthStoreRecovery::certification_recover_with_custody_pauses(
                worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(&root),
                |_| {},
                || {},
            );
            let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("genuine selected tier custody failed Store rejoin: {outcome:?}")
            };
            let seal = handoff
                .into_core()
                .certification_release_media_into_checkpoint_custody()
                .expect("selected tier custody seal");
            worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal(&root, seal);
        })
        .expect("tier recovery worker");
    worker.join().expect("tier recovery worker did not panic");
}
