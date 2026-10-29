//! A checkpoint capture runs on the Store's standing reservation at a
//! realistic head population: it takes no new Recovery bytes, and its real
//! heap allocations fit the envelope that reservation holds. The child also reports the process heap peak; the
//! dirty-frame slices, media and signal work it includes are separately owned
//! and outside this envelope.

#[allow(unused_imports)] use worth_store_testkit_store::physical_blob_journeys::checkpoint_capture_envelope::*;
use {std::fs, std::num::NonZeroU16, std::num::NonZeroU64, std::path::Path, std::process::Command};

use {worth_proof::AdmittedBlobReleaseProof, worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::production::PhysicalCheckpointStep};
use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimReceipt, worth_store::physical_runtime::BlobReclaimRequest, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::ServingPhysicalRuntime};
use {worth_store_blob_chunks::BlobChunkSize};

use {worth_store_testkit_store::allocation_probe::physical_blob_journeys::attribute_allocations, worth_store_testkit_store::allocation_probe::physical_blob_journeys::attributed_live_bytes, worth_store_testkit_store::allocation_probe::physical_blob_journeys::peak_live_bytes_during};
use {worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization};

#[test]
fn checkpoint_capture_runs_on_its_standing_reservation_and_its_heap_fits_the_envelope() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    std::fs::create_dir(&root).unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
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
