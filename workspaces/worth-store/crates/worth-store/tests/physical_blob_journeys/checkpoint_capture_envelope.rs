//! A checkpoint capture runs on the Store's standing reservation at a
//! realistic head population: it takes no new Recovery bytes, and its real
//! heap allocations fit the envelope that reservation holds. The child also reports the process heap peak; the
//! dirty-frame slices, media and signal work it includes are separately owned
//! and outside this envelope.

use std::{
    fs,
    num::{NonZeroU16, NonZeroU64},
    path::Path,
    process::Command,
};

use worth_proof::{AdmittedBlobReleaseProof, TransitionOutcome};
use worth_store::physical_runtime::production::PhysicalCheckpointStep;
use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits, BlobReclaimDisposition,
    BlobReclaimLimits, BlobReclaimReceipt, BlobReclaimRequest, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
    PhysicalMutationDeadline, PhysicalOperationAllocationScope, ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;

use super::allocation_probe::{
    attribute_allocations, attributed_live_bytes, peak_live_bytes_during,
};
use super::fixture::{admitted_blob_scope, placement, serving_from_initialization};

const CHUNK: usize = 64 << 10;
/// Distinct partially released objects, one keyed head each. Every drop but
/// the last is checkpointed; the last leaves its Batch pending for capture.
const HEADS: u8 = 16;
const REPORT: &str = "C11_CHECKPOINT_ENVELOPE ";

#[test]
fn checkpoint_capture_runs_on_its_standing_reservation_and_its_heap_fits_the_envelope() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    fs::create_dir(&root).unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "c11_blob_child_role",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("WORTH_STORE_C11_BLOB_CHILD_ROLE", "checkpoint-envelope")
        .env("WORTH_STORE_C11_BLOB_CHILD_ROOT", &root)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains(REPORT),
        "envelope child failed
stdout:
{stdout}
stderr:
{}",
        String::from_utf8_lossy(&output.stderr)
    );
    if let Some(line) = stdout.lines().find(|line| line.contains(REPORT)) {
        eprintln!("{line}");
    }
}

/// Runs in its own process: the allocation probe observes every thread.
pub(super) fn child(root: &Path) {
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
            serving.pause_physical_checkpoint_at(PhysicalCheckpointStep::CandidateSynchronization);
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
        matches!(outcome, PhysicalCheckpointOutcome::Completed(_)),
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

pub(super) fn release_one_batch(
    serving: &ServingPhysicalRuntime,
    seed: u8,
    keep_pending: bool,
) -> BlobReclaimReceipt {
    let scope = admitted_blob_scope("c11.blob.checkpoint.envelope.scope");
    let read_limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
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
    let proof = AdmittedBlobReleaseProof::certification_admit(
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
        .reclaim(BlobReclaimRequest::released(
            proof,
            placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            BlobReclaimLimits::new(
                NonZeroU64::new(256).unwrap(),
                NonZeroU64::new(64 << 20).unwrap(),
                NonZeroU16::new(1).unwrap(),
            )
            .unwrap(),
        ))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
    receipt
}

fn recovery_bytes(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .residency_observation()
        .counters()
        .active_operation_bytes_for(PhysicalOperationAllocationScope::Recovery)
}

fn request(key: u8) -> PhysicalCheckpointRequest {
    PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    )
}

fn start(
    serving: &ServingPhysicalRuntime,
    key: u8,
) -> worth_store::physical_runtime::PhysicalCheckpointHandle {
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request(key)).into_raw()
    else {
        panic!("the capture envelope must admit within the Recovery budget")
    };
    handle
}
