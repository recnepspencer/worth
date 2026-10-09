use std::num::NonZeroU64;

use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobIngestFailure, BlobMemoryDenial,
    BlobReadLimits, PhysicalMutationDeadline, PhysicalOperationAllocationScope,
};
use worth_store_blob_chunks::BlobChunkSize;

use super::fixture::{admitted_blob_scope, placement, serving_from_initialization};

#[test]
fn source_window_and_frames_are_denied_before_chunk_effects() {
    const CHUNK: usize = 64 * 1024;
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let blob_bytes = || {
        serving
            .residency_observation()
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Blob)
    };
    assert_eq!(blob_bytes(), 0);
    let blobs = serving.blobs().unwrap();
    let scope = admitted_blob_scope("c11.blob.admission.scope");
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = || {
        BlobIngestDeclaration::new(
            object,
            BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
            (2 * CHUNK) as u64,
            &scope,
            BlobCheckpointLimit::bounded_horizon(16).unwrap(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        )
        .unwrap()
    };
    assert!(matches!(
        blobs.begin_ingest(declaration(), placement(), (2 * CHUNK) as u64, limits),
        Err(BlobIngestFailure::Memory(
            BlobMemoryDenial::WindowCoversObject
        ))
    ));
    assert_eq!(blob_bytes(), 0, "denied admission must release its charge");

    let mut ingest = blobs
        .begin_ingest(declaration(), placement(), CHUNK as u64, limits)
        .unwrap();
    let ingest_charge = (CHUNK + 5 * 1024 * 1024) as u64;
    assert_eq!(blob_bytes(), ingest_charge);
    let session = ingest.session_id();
    let payload = vec![91_u8; 2 * CHUNK];
    assert!(matches!(
        ingest.push(&payload),
        Err(BlobIngestFailure::SourceFrameExceedsWindow {
            supplied,
            admitted_window,
        }) if supplied == payload.len() as u64 && admitted_window == CHUNK as u64
    ));
    assert!(matches!(
        ingest.push(&payload[..CHUNK + 1]),
        Err(BlobIngestFailure::SourceFrameExceedsWindow { .. })
    ));
    assert!(ingest.memory_observation().peak_charged_bytes() <= CHUNK as u64 + 5 * 1024 * 1024);
    ingest.push(&payload[..CHUNK]).unwrap();
    assert_eq!(
        blob_bytes(),
        ingest_charge,
        "effects retain the Blob charge"
    );
    ingest.push(&payload[CHUNK..]).unwrap();
    let published = ingest.finish().unwrap();
    assert_eq!(blob_bytes(), 0, "publication releases the ingest charge");
    assert_eq!(published.session(), session);
    let resolved = blobs
        .resolve_publication(object.bytes(), 1, &scope, limits)
        .unwrap();
    assert_eq!(resolved, published);
    drop(blobs);
    serving.close();
}
