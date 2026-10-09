//! A completed partial drop leaves a standing checkpoint reservation. Its
//! checkpoint runs on that reservation even when every other Recovery byte
//! is held, and a cancelled capture leaves the reservation whole.

use super::*;
use worth_store::physical_runtime::production::PhysicalCheckpointStep;
use worth_store::physical_runtime::{
    BlobReclaimRetirement, PhysicalCheckpointCancellationOutcome, PhysicalCheckpointHandle,
    PhysicalCheckpointProvenNoEffectCause, PhysicalRetirementDenial, ServingPhysicalRuntime,
};

#[test]
fn partial_drop_checkpoints_on_its_standing_reservation_and_reopens() {
    std::thread::Builder::new()
        .name("release-checkpoint-reservation".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("checkpoint reservation worker")
        .join()
        .expect("checkpoint reservation worker did not panic");
}

fn run() {
    // One head closure for the release certificate, plus the drop's capture
    // custody (its envelope carries the policy's pin-scan bound, about 2.3 MB
    // here) and headroom.
    let limit = numeric_one_head_closure() + (4 << 20);
    let (world, proof, _) =
        release_reopen::published_world::create(false, None, NonZeroU64::new(limit));
    let source = (proof.object(), proof.generation());
    let serving = world.serving();
    let selected_before_drop = checkpoint_family(world.root());
    // Reclaim normally checkpoints its drop before returning. Fail that real
    // scheduler admission so the completed drop still needs its first checkpoint.
    serving.certification_fail_next_checkpoint_admission();
    let request = BlobReclaimRequest::released(
        proof,
        world.placement(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        BlobReclaimLimits::new(
            NonZeroU64::new(1024).unwrap(),
            NonZeroU64::new(64 << 20).unwrap(),
            NonZeroU16::new(1).unwrap(),
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
    assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
    assert_eq!(receipt.dropped_records().len(), 1);
    assert!(receipt.remaining_payload_records() > 0);
    assert_eq!(
        receipt.retirement(),
        BlobReclaimRetirement::Pending(PhysicalRetirementDenial::Checkpoint)
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
            PhysicalOperationAllocationScope::Recovery,
            NonZeroU64::new(held_bytes).unwrap(),
        )
        .expect("competing real Recovery allocation after the completed drop");
    assert_eq!(recovery_bytes(serving), limit - 1);

    // Every other Recovery byte is held: the capture consumes the reservation.
    let selected_before = snapshot_family(world.root());
    let gate =
        serving.pause_physical_checkpoint_at(PhysicalCheckpointStep::CandidateSynchronization);
    let handle = start(serving, 0x7b);
    assert!(
        gate.await_arrival(),
        "a capture on the reservation must reach file synchronization"
    );
    assert_eq!(recovery_bytes(serving), limit - 1, "no fresh capture bytes");
    assert!(matches!(
        handle.request_cancellation(),
        PhysicalCheckpointCancellationOutcome::Accepted { .. }
    ));
    gate.release();
    match handle.wait() {
        PhysicalCheckpointOutcome::ProvenNoEffect(no_effect) => {
            assert_eq!(
                no_effect.cause(),
                PhysicalCheckpointProvenNoEffectCause::CancelledAndCandidateRemoved
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
        matches!(completed, PhysicalCheckpointOutcome::Completed(_)),
        "the retry runs on the same reservation under held pressure: {completed:?}"
    );
    // The committed roster lives inside the reservation that funded it.
    drop(held);
    assert_eq!(recovery_bytes(serving), retained);

    let checkpoint = fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let (batches, accumulator) =
        release_reopen::selected_release_certificates_from_bytes(&checkpoint);
    let [batch] = batches.as_slice() else {
        panic!("one genuine drop must produce one selected Batch");
    };
    assert_eq!(batch.cumulative_dropped(), 1);
    assert!(!batch.terminal());
    let heads = release_reopen::selected_head_oracle::selected_heads(world.root(), accumulator);
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
    super::super::recover(root, super::super::Mutation::None);
}

fn checkpoint_request(key: u8) -> PhysicalCheckpointRequest {
    PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    )
}

fn start(serving: &ServingPhysicalRuntime, key: u8) -> PhysicalCheckpointHandle {
    match serving
        .checkpoints()
        .start(checkpoint_request(key))
        .into_raw()
    {
        TransitionOutcome::Success(handle) => handle,
        TransitionOutcome::Failed(failure) => {
            panic!("checkpoint {key:#x} failed within the Recovery budget: {failure:?}")
        }
        _ => panic!("checkpoint must admit within the Recovery budget"),
    }
}

fn recovery_bytes(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .residency_observation()
        .counters()
        .active_operation_bytes_for(PhysicalOperationAllocationScope::Recovery)
}

fn checkpoint_family(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
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
