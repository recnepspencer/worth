use {std::num::NonZeroU64};

use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobMemoryDenial, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalOperationAllocationScope};
use {worth_store_blob_chunks::BlobChunkSize};

use {worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization};

#[test]
fn source_window_and_frames_are_denied_before_chunk_effects() {
    const CHUNK: usize = 64 * 1024;
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let blob_bytes = || {
        serving
            .residency_observation()
            .counters()
            .active_operation_bytes_for(worth_store::physical_runtime::PhysicalOperationAllocationScope::Blob)
    };
    assert_eq!(blob_bytes(), 0);
    let blobs = serving.blobs().unwrap();
    let scope = admitted_blob_scope("c11.blob.admission.scope");
    let limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = || {
        worth_store::physical_runtime::BlobIngestDeclaration::new(
            object,
            worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
            (2 * CHUNK) as u64,
            &scope,
            worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        )
        .unwrap()
    };
    assert!(matches!(
        blobs.begin_ingest(declaration(), placement(), (2 * CHUNK) as u64, limits),
        Err(worth_store::physical_runtime::BlobIngestFailure::Memory(
            worth_store::physical_runtime::BlobMemoryDenial::WindowCoversObject
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
        Err(worth_store::physical_runtime::BlobIngestFailure::SourceFrameExceedsWindow {
            supplied,
            admitted_window,
        }) if supplied == payload.len() as u64 && admitted_window == CHUNK as u64
    ));
    assert!(matches!(
        ingest.push(&payload[..CHUNK + 1]),
        Err(worth_store::physical_runtime::BlobIngestFailure::SourceFrameExceedsWindow { .. })
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
    serving.close();
}
