//! The C.8 rejoin hands Serving a ledger holding released-drop custody. Entry
//! funds its standing capture reservation before Serving is sealed, so a
//! foreign Recovery hold taken after entry, leaving the pool one byte, cannot
//! starve the checkpoint that must certify the custody.

use std::num::NonZeroU64;

use worth_store::physical_runtime::{RecoveryPhysicalAllocation, ServingPhysicalRuntime};
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

use super::*;

#[test]
fn rejoined_release_custody_checkpoints_under_a_foreign_hold() {
    let (world, _, _) = release_reopen::released_world(1024);
    checkpoint(world.serving(), [0xc1; 32]);
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);

    let worker = std::thread::Builder::new()
        .name("rejoined-release-custody-liveness".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let outcome = WorthStoreRecovery::recover(super::recovery_request::request(&root));
            let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("genuine selected release recovery denied: {outcome:?}")
            };
            let seal = handoff
                .into_core()
                .into_checkpoint_custody()
                .expect("selected release grants a one-shot custody seal");
            let serving = super::admit_serving_with_seal(&root, seal);
            assert!(
                serving
                    .certification_checkpoint_capture_custody_bytes()
                    .is_some(),
                "Serving entry funds the rejoined custody's capture reservation"
            );
            let hold = hold_all_but_one_byte(&serving);
            checkpoint(&serving, [0xc2; 32]);
            drop(hold);
            assert!(!serving.close().residency().requires_inspection());
        })
        .expect("rejoin liveness worker");
    worker.join().expect("rejoin liveness worker did not panic");
}

/// Holds, in a foreign Recovery grant, every byte the carried pool would still
/// grant except one. The bound is found against the real pool.
fn hold_all_but_one_byte(serving: &ServingPhysicalRuntime) -> RecoveryPhysicalAllocation<'_> {
    let issuer = serving.physical_allocations();
    let admit = |bytes: u64| issuer.admit_recovery(NonZeroU64::new(bytes).unwrap());
    let (mut granted, mut denied) = (1_u64, 1_u64 << 48);
    assert!(admit(granted).is_ok() && admit(denied).is_err());
    while denied - granted > 1 {
        let middle = granted + (denied - granted) / 2;
        if admit(middle).is_ok() {
            granted = middle;
        } else {
            denied = middle;
        }
    }
    let hold = admit(granted - 1).expect("the foreign hold fits the pool");
    assert!(admit(2).is_err(), "the hold leaves the pool one byte");
    hold
}

fn checkpoint(serving: &ServingPhysicalRuntime, key: [u8; 32]) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let handle = match serving.checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        TransitionOutcome::Failed(failure) => {
            panic!("checkpoint must admit under a foreign hold: {failure:?}")
        }
        _ => panic!("checkpoint must admit under a foreign hold"),
    };
    let outcome = handle.wait();
    assert!(
        matches!(outcome, PhysicalCheckpointOutcome::Completed(_)),
        "checkpoint must complete under a foreign hold: {outcome:?}"
    );
}
