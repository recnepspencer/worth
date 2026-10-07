use std::num::NonZeroU64;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, RecordAppendBatch,
    RecordByteLimit, RecordCountLimit, RecordReadLimits, RecordScanOutcome, RecordScanRequest,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::PersistedRecordIdentity;

use super::fixture::{admitted_blob_scope, placement, serving_from_initialization};

const CHUNK: usize = 64 * 1024;

#[test]
fn blob_append_releases_only_its_own_clean_frames() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let placement = placement();
    let ordinary = vec![0x74; CHUNK];
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0xa7; 32]))
        .unwrap();
    let prepared = match submission
        .prepare_durable_append(
            RecordAppendBatch::try_from_iter([ordinary.as_slice()]).unwrap(),
            placement,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("ordinary extent preparation must admit"),
    };
    let persisted = match prepared.execute() {
        PhysicalMutationOutcome::Completed(completed) => completed.persisted_records()[0],
        _ => panic!("ordinary extent must durably publish"),
    };
    let record = selected_record_id(&serving, persisted);

    let blobs = serving.blobs().unwrap();
    let scope = admitted_blob_scope("c11.blob.cache.retention");
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (4 * CHUNK) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
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
    drop(blobs);
    serving.close();
}

fn selected_record_id(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    persisted: PersistedRecordIdentity,
) -> worth_store::physical_runtime::PhysicalRecordId {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(16).unwrap())
                .with_payload_limit(RecordByteLimit::new(1).unwrap()),
        )
        .unwrap();
    let mut scratch = [0_u8; 256];
    match scan.read_next_into(&mut scratch).unwrap() {
        RecordScanOutcome::Batch(batch) => batch
            .records()
            .iter()
            .find(|record| {
                record.record_id().allocation_epoch() == persisted.allocation_epoch()
                    && record.record_id().ordinal() == persisted.ordinal()
            })
            .expect("selected ordinary record")
            .record_id(),
        RecordScanOutcome::Completed(_) => panic!("ordinary record vanished from selected root"),
    }
}

fn read_ordinary(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    record: worth_store::physical_runtime::PhysicalRecordId,
) -> Vec<u8> {
    read_ordinary_observed(serving, record).0
}

fn read_ordinary_observed(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    record: worth_store::physical_runtime::PhysicalRecordId,
) -> (Vec<u8>, u64) {
    let mut read = serving
        .records()
        .unwrap()
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(CHUNK as u32).unwrap()),
        )
        .unwrap();
    let mut bytes = vec![0_u8; CHUNK];
    let mut completed = 0;
    while completed < bytes.len() {
        let count = read.read_next(&mut bytes[completed..]).unwrap();
        assert!(count != 0);
        completed += count;
    }
    assert_eq!(read.read_next(&mut [0_u8; 1]).unwrap(), 0);
    (bytes, read.observation().physical_work_count())
}
