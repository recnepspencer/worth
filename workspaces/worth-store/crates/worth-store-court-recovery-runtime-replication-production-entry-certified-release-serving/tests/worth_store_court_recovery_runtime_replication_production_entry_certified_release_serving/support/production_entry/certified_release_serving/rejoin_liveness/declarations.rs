//! The C.8 rejoin hands Serving a ledger holding released-drop custody. Entry
//! funds its standing capture reservation before Serving is sealed, so a
//! foreign Recovery hold taken after entry, leaving the pool one byte, cannot
//! starve the checkpoint that must certify the custody.

pub(crate) use std::num::NonZeroU64;

pub(crate) use {
    worth_store::physical_runtime::RecoveryPhysicalAllocation,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

pub(crate) use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request, worth_store_testkit_recovery_runtime::production_entry::release_reopen, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request};

/// Holds, in a foreign Recovery grant, every byte the carried pool would still
/// grant except one. The bound is found against the real pool.
pub(crate) fn hold_all_but_one_byte(
    serving: &ServingPhysicalRuntime,
) -> RecoveryPhysicalAllocation<'_> {
    let issuer = serving.physical_allocations();
    let admit = |bytes: u64| issuer.admit_recovery(std::num::NonZeroU64::new(bytes).unwrap());
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

pub(crate) fn checkpoint(serving: &ServingPhysicalRuntime, key: [u8; 32]) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(key),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let handle = match serving.checkpoints().start(request).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        worth_proof::TransitionOutcome::Failed(failure) => {
            panic!("checkpoint must admit under a foreign hold: {failure:?}")
        }
        _ => panic!("checkpoint must admit under a foreign hold"),
    };
    let outcome = handle.wait();
    assert!(
        matches!(
            outcome,
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
        ),
        "checkpoint must complete under a foreign hold: {outcome:?}"
    );
}
