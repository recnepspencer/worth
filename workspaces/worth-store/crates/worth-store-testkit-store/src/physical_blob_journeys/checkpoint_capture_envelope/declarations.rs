//! A checkpoint capture runs on the Store's standing reservation at a
//! realistic head population: it takes no new Recovery bytes, and its real
//! heap allocations fit the envelope that reservation holds. The child also reports the process heap peak; the
//! dirty-frame slices, media and signal work it includes are separately owned
//! and outside this envelope.

pub use {std::fs, std::num::NonZeroU16, std::num::NonZeroU64, std::path::Path, std::process::Command};

pub use {worth_proof::AdmittedBlobReleaseProof, worth_proof::TransitionOutcome};
pub use {worth_store::physical_runtime::production::PhysicalCheckpointStep};
pub use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimReceipt, worth_store::physical_runtime::BlobReclaimRequest, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::ServingPhysicalRuntime};
pub use {worth_store_blob_chunks::BlobChunkSize};

pub use {crate::allocation_probe::physical_blob_journeys::attribute_allocations, crate::allocation_probe::physical_blob_journeys::attributed_live_bytes, crate::allocation_probe::physical_blob_journeys::peak_live_bytes_during};
pub use {crate::fixture::admitted_blob_scope, crate::fixture::placement, crate::fixture::serving_from_initialization};

pub const CHUNK: usize = 64 << 10;
/// Distinct partially released objects, one keyed head each. Every drop but
/// the last is checkpointed; the last leaves its Batch pending for capture.
pub const HEADS: u8 = 16;
pub const REPORT: &str = "C11_CHECKPOINT_ENVELOPE ";

/// Runs in its own process: the allocation probe observes every thread.
pub fn child(root: &Path) {
    let serving = serving_from_initialization(root);
    for object in 0..HEADS {
        release_one_batch(&serving, object, object + 1 == HEADS);
    }
    let envelope = serving
        .certification_checkpoint_capture_envelope_bytes()
        .expect("selected release custody has a capture envelope");
    let before = recovery_bytes(&serving);
    let ((outcome, during, attributed), peak) = peak_live_bytes_during(|| {
        let gate =
            serving.pause_physical_checkpoint_at(worth_store::physical_runtime::production::PhysicalCheckpointStep::CandidateSynchronization);
        let handle = attribute_allocations(|| start(&serving, 0x7b));
        assert!(
            gate.await_arrival(),
            "capture must reach file synchronization"
        );
        let during = recovery_bytes(&serving);
        let attributed = attributed_live_bytes() as u64;
        gate.release();
        (handle.wait(), during, attributed)
    });
    assert!(
        matches!(outcome, worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)),
        "the capture must complete on its standing reservation"
    );
    println!(
        "{REPORT}heads={HEADS} envelope={envelope} attributed={attributed} measured_peak={peak}"
    );
    assert_eq!(during, before, "the capture took new Recovery bytes");
    // Every byte the capture call left live on its thread at the pause: the
    // envelope's prepared buffers and sealed storage, plus the runtime's own
    // attempt and worker records. The worker's commands are not on this
    // thread. Even with those records, the total fits the one envelope.
    assert!(
        attributed <= envelope,
        "the capture's live allocations ({attributed}) exceed its envelope ({envelope})"
    );
    serving.close();
}

pub fn release_one_batch(
    serving: &ServingPhysicalRuntime,
    seed: u8,
    keep_pending: bool,
) -> BlobReclaimReceipt {
    let scope = admitted_blob_scope("c11.blob.checkpoint.envelope.scope");
    let read_limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK as u64, read_limits)
        .unwrap();
    ingest.push(&vec![seed; CHUNK]).unwrap();
    ingest.push(&vec![!seed; CHUNK]).unwrap();
    let published = ingest.finish().unwrap();
    let marker = serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("published generation has a selected marker");
    let record = marker.record();
    let proof = worth_proof::AdmittedBlobReleaseProof::certification_admit(
        serving.store_identity().bytes(),
        object.bytes(),
        published.generation().sequence(),
        record.allocation_epoch(),
        record.ordinal(),
        marker.encoded_digest(),
        [0x71; 32],
    )
    .unwrap();
    if keep_pending {
        // Keep the drop's Batch pending for the measured capture.
        serving.certification_fail_next_checkpoint_admission();
    }
    let receipt = blobs
        .reclaim(worth_store::physical_runtime::BlobReclaimRequest::released(
            proof,
            placement(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            worth_store::physical_runtime::BlobReclaimLimits::new(
                std::num::NonZeroU64::new(256).unwrap(),
                std::num::NonZeroU64::new(64 << 20).unwrap(),
                std::num::NonZeroU16::new(1).unwrap(),
            )
            .unwrap(),
        ))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(receipt.disposition(), worth_store::physical_runtime::BlobReclaimDisposition::Dropped);
    receipt
}

pub fn recovery_bytes(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .residency_observation()
        .counters()
        .active_operation_bytes_for(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery)
}

pub fn request(key: u8) -> PhysicalCheckpointRequest {
    worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    )
}

pub fn start(
    serving: &ServingPhysicalRuntime,
    key: u8,
) -> worth_store::physical_runtime::PhysicalCheckpointHandle {
    let worth_proof::TransitionOutcome::Success(handle) = serving.checkpoints().start(request(key)).into_raw()
    else {
        panic!("the capture envelope must admit within the Recovery budget")
    };
    handle
}