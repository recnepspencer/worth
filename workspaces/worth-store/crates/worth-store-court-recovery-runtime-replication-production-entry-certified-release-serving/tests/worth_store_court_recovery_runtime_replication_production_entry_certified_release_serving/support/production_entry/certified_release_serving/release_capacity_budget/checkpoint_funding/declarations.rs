//! A completed partial drop leaves a standing checkpoint reservation. Its
//! checkpoint runs on that reservation even when every other Recovery byte
//! is held, and a cancelled capture leaves the reservation whole.

pub(crate) use {worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation, std::num::NonZeroU16, std::num::NonZeroU64, std::path::Path, std::path::PathBuf, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_proof::TransitionOutcome, std::fs, crate::production_entry::certified_release_serving::release_capacity_budget::funded_one_head_limit, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover, worth_store_testkit_recovery_runtime::production_entry::release_reopen, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request, crate::production_entry::certified_release_serving::release_capacity_budget::snapshot_family};
pub(crate) use {worth_store::physical_runtime::production::PhysicalCheckpointStep};
pub(crate) use {worth_store::physical_runtime::BlobReclaimRetirement, worth_store::physical_runtime::PhysicalCheckpointCancellationOutcome, worth_store::physical_runtime::PhysicalCheckpointHandle, worth_store::physical_runtime::PhysicalCheckpointProvenNoEffectCause, worth_store::physical_runtime::PhysicalRetirementDenial, worth_store::physical_runtime::ServingPhysicalRuntime};

pub(crate) fn run() {
    let limit = funded_one_head_limit();
    let (world, proof, _) =
        worth_store_testkit_recovery_runtime::production_entry::release_reopen::published_world::create(false, None, std::num::NonZeroU64::new(limit));
    let source = (proof.object(), proof.generation());
    let serving = world.serving();
    let selected_before_drop = checkpoint_family(world.root());
    // Reclaim normally checkpoints its drop before returning. Fail that real
    // scheduler admission so the completed drop still needs its first checkpoint.
    serving.certification_fail_next_checkpoint_admission();
    let request = worth_store::physical_runtime::BlobReclaimRequest::released(
        proof,
        world.placement(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
            .unwrap(),
        worth_store::physical_runtime::BlobReclaimLimits::new(
            std::num::NonZeroU64::new(1024).unwrap(),
            std::num::NonZeroU64::new(64 << 20).unwrap(),
            std::num::NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    );
    let receipt = serving
        .blobs()
        .unwrap()
        .reclaim(request)
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(
        receipt.disposition(),
        worth_store::physical_runtime::BlobReclaimDisposition::Dropped
    );
    assert_eq!(receipt.dropped_records().len(), 1);
    assert!(receipt.remaining_payload_records() > 0);
    assert_eq!(
        receipt.retirement(),
        worth_store::physical_runtime::BlobReclaimRetirement::Pending(
            worth_store::physical_runtime::PhysicalRetirementDenial::Checkpoint
        )
    );
    assert_eq!(checkpoint_family(world.root()), selected_before_drop);

    // The drop's retained custody includes the reservation for its checkpoint.
    let retained = recovery_bytes(serving);
    assert!(retained > 0, "drop retains real Recovery custody");
    let counters = serving.residency_observation().counters();
    let held_bytes = limit - retained - 1;
    assert!(32 << 20 > counters.active_operation_bytes() + held_bytes + (1 << 20));
    let held = serving
        .certification_physical_residency()
        .admit_operation_scope(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery,
            std::num::NonZeroU64::new(held_bytes).unwrap(),
        )
        .expect("competing real Recovery allocation after the completed drop");
    assert_eq!(recovery_bytes(serving), limit - 1);

    // Every other Recovery byte is held: the capture consumes the reservation.
    let selected_before = snapshot_family(world.root());
    let gate = serving.pause_physical_checkpoint_at(
        worth_store::physical_runtime::production::PhysicalCheckpointStep::CandidateSynchronization,
    );
    let handle = start(serving, 0x7b);
    // The pause probe waits five seconds per call; the admitted checkpoint has
    // thirty seconds. Observe that whole envelope without restarting its work.
    assert!(
        (0..6).any(|_| gate.await_arrival()),
        "a capture on the reservation must reach file synchronization"
    );
    assert_eq!(recovery_bytes(serving), limit - 1, "no fresh capture bytes");
    assert!(matches!(
        handle.request_cancellation(),
        worth_store::physical_runtime::PhysicalCheckpointCancellationOutcome::Accepted { .. }
    ));
    gate.release();
    match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::ProvenNoEffect(no_effect) => {
            assert_eq!(
                no_effect.cause(),
                worth_store::physical_runtime::PhysicalCheckpointProvenNoEffectCause::CancelledAndCandidateRemoved
            );
        }
        other => {
            panic!("cancelled candidate must be removed without losing drop custody: {other:?}")
        }
    }
    assert_eq!(snapshot_family(world.root()), selected_before);
    assert_eq!(
        recovery_bytes(serving),
        limit - 1,
        "a cancelled capture leaves the reservation whole"
    );
    let completed = start(serving, 0x7c).wait();
    assert!(
        matches!(
            completed,
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
        ),
        "the retry runs on the same reservation under held pressure: {completed:?}"
    );
    // The committed roster lives inside the reservation that funded it.
    drop(held);
    assert_eq!(recovery_bytes(serving), retained);

    let checkpoint = std::fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let (batches, accumulator) =
        worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates_from_bytes(&checkpoint);
    let [batch] = batches.as_slice() else {
        panic!("one genuine drop must produce one selected Batch");
    };
    assert_eq!(batch.cumulative_dropped(), 1);
    assert!(!batch.terminal());
    let heads =
        worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(
            world.root(),
            accumulator,
        );
    let [head] = heads.as_slice() else {
        panic!("one partial release must retain its keyed head");
    };
    assert_eq!(head.key().object(), source.0);
    assert_eq!(head.key().generation(), source.1);
    assert_eq!(head.cumulative_dropped(), 1);
    assert!(!head.terminal());
    let retained_root = world.retained_root();
    let root = retained_root.path().to_path_buf();
    drop(world);
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover(root, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation::None);
}

pub(crate) fn checkpoint_request(key: u8) -> PhysicalCheckpointRequest {
    worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    )
}

pub(crate) fn start(serving: &ServingPhysicalRuntime, key: u8) -> PhysicalCheckpointHandle {
    match serving
        .checkpoints()
        .start(checkpoint_request(key))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        worth_proof::TransitionOutcome::Failed(failure) => {
            panic!("checkpoint {key:#x} failed within the Recovery budget: {failure:?}")
        }
        _ => panic!("checkpoint must admit within the Recovery budget"),
    }
}

pub(crate) fn recovery_bytes(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .residency_observation()
        .counters()
        .active_operation_bytes_for(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery,
        )
}

pub(crate) fn checkpoint_family(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    snapshot_family(root)
        .into_iter()
        .filter(|(path, _)| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("checkpoint")
        })
        .collect()
}
