#[allow(unused_imports)] use crate::physical_blob_journeys::cache_retention::*;
use {std::num::NonZeroU64};

use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest};
use {worth_store_blob_chunks::BlobChunkSize};
use {worth_store_physical_format::PersistedRecordIdentity};

use {worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization};

#[test]
fn blob_append_releases_only_its_own_clean_frames() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let placement = placement();
    let ordinary = vec![0x74; CHUNK];
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0xa7; 32]))
        .unwrap();
    let prepared = match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([ordinary.as_slice()]).unwrap(),
            placement,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("ordinary extent preparation must admit"),
    };
    let persisted = match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(completed) => completed.persisted_records()[0],
        _ => panic!("ordinary extent must durably publish"),
    };
    let record = selected_record_id(&serving, persisted);

    let blobs = serving.blobs().unwrap();
    let scope = admitted_blob_scope("c11.blob.cache.retention");
    let limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (4 * CHUNK) as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement, CHUNK as u64, limits)
        .unwrap();

    assert_eq!(read_ordinary(&serving, record), ordinary);
    assert_eq!(read_ordinary(&serving, record), ordinary);
    let warmed_bytes = serving.residency_observation().counters().resident_bytes();
    for index in 0..4 {
        ingest.push(&vec![index as u8; CHUNK]).unwrap();
        let (bytes, physical_work) = read_ordinary_observed(&serving, record);
        assert_eq!(bytes, ordinary);
        assert_eq!(
            physical_work, 0,
            "blob append evicted the unrelated warm ordinary extent"
        );
    }
    let after = serving.residency_observation().counters().resident_bytes();
    assert!(
        after <= warmed_bytes + 2 * CHUNK as u64,
        "blob candidates accumulated in shared residency: {warmed_bytes} -> {after}"
    );
    ingest.finish().unwrap();
    serving.close();
}
