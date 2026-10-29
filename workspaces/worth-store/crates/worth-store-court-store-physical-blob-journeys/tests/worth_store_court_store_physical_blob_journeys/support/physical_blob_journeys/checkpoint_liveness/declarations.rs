//! Checkpoint liveness against foreign pool holds. A checkpoint draws only on
//! the standing capture reservation, so an admitted drop's checkpoint, and the
//! retirement waiting on it, still progress when another owner holds every
//! byte the pool would grant to an ordinary scope but one. Ordinary reopen
//! never holds released-drop custody (it requires C.8), so the entry
//! reservation is proven on the C.8 rejoin in production_entry.

pub(crate) use {std::num::NonZeroU32, std::num::NonZeroU64};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::BlobReclaimRetirement, worth_store::physical_runtime::BlobReclaimRetirementBudget, worth_store::physical_runtime::CertificationScopedAllocation, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::PhysicalRetirementDenial, worth_store::physical_runtime::ServingPhysicalRuntime};

pub(crate) use {worth_store_testkit_store::physical_blob_journeys::checkpoint_capture_envelope::release_one_batch};
pub(crate) use {worth_store_testkit_store::fixture::serving_from_initialization};

/// Holds, in a foreign ordinary scope, every operation byte the pool would
/// still grant except one. The bound is found against the real pool.
pub(crate) fn hold_all_but_one_byte(serving: &ServingPhysicalRuntime) -> CertificationScopedAllocation {
    let residency = serving.certification_physical_residency();
    let admit = |bytes: u64| {
        residency.admit_operation_scope(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundRead,
            std::num::NonZeroU64::new(bytes).unwrap(),
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

pub(crate) fn checkpoint(serving: &ServingPhysicalRuntime, key: u8) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let handle = match serving.checkpoints().start(request).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        worth_proof::TransitionOutcome::Failed(failure) => {
            panic!("checkpoint {key:#x} must admit under a foreign hold: {failure:?}")
        }
        _ => panic!("checkpoint {key:#x} must admit under a foreign hold"),
    };
    let outcome = handle.wait();
    assert!(
        matches!(outcome, worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)),
        "checkpoint {key:#x} must complete under a foreign hold: {outcome:?}"
    );
}

pub(crate) fn retirement_budget() -> BlobReclaimRetirementBudget {
    worth_store::physical_runtime::BlobReclaimRetirementBudget::new(
        std::num::NonZeroU32::new(2).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
}