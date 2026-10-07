//! Checkpoint liveness against foreign pool holds. A checkpoint draws only on
//! the standing capture reservation, so an admitted drop's checkpoint, and the
//! retirement waiting on it, still progress when another owner holds every
//! byte the pool would grant to an ordinary scope but one. Ordinary reopen
//! never holds released-drop custody (it requires C.8), so the entry
//! reservation is proven on the C.8 rejoin in production_entry.

use std::num::{NonZeroU32, NonZeroU64};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    BlobReclaimRetirement, BlobReclaimRetirementBudget, CertificationScopedAllocation,
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, PhysicalMutationDeadline, PhysicalOperationAllocationScope,
    PhysicalRetirementDenial, ServingPhysicalRuntime,
};

use super::checkpoint_capture_envelope::release_one_batch;
use super::fixture::serving_from_initialization;

#[test]
fn admitted_drop_checkpoints_and_retires_under_a_foreign_hold() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let mut receipt = release_one_batch(&serving, 0x31, true);
    assert_eq!(
        receipt.retirement(),
        BlobReclaimRetirement::Pending(PhysicalRetirementDenial::Checkpoint)
    );
    let hold = hold_all_but_one_byte(&serving);
    checkpoint(&serving, 0x91);
    let retirement = serving
        .blobs()
        .unwrap()
        .continue_reclaim_retirement(&mut receipt, retirement_budget())
        .unwrap();
    assert_ne!(
        retirement,
        BlobReclaimRetirement::Pending(PhysicalRetirementDenial::Checkpoint),
        "the committed checkpoint must release the drop's retirement"
    );
    drop(hold);
    serving.close();
}

/// Holds, in a foreign ordinary scope, every operation byte the pool would
/// still grant except one. The bound is found against the real pool.
fn hold_all_but_one_byte(serving: &ServingPhysicalRuntime) -> CertificationScopedAllocation {
    let residency = serving.certification_physical_residency();
    let admit = |bytes: u64| {
        residency.admit_operation_scope(
            PhysicalOperationAllocationScope::ForegroundRead,
            NonZeroU64::new(bytes).unwrap(),
        )
    };
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

fn checkpoint(serving: &ServingPhysicalRuntime, key: u8) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let handle = match serving.checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        TransitionOutcome::Failed(failure) => {
            panic!("checkpoint {key:#x} must admit under a foreign hold: {failure:?}")
        }
        _ => panic!("checkpoint {key:#x} must admit under a foreign hold"),
    };
    let outcome = handle.wait();
    assert!(
        matches!(outcome, PhysicalCheckpointOutcome::Completed(_)),
        "checkpoint {key:#x} must complete under a foreign hold: {outcome:?}"
    );
}

fn retirement_budget() -> BlobReclaimRetirementBudget {
    BlobReclaimRetirementBudget::new(
        NonZeroU32::new(2).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
}
