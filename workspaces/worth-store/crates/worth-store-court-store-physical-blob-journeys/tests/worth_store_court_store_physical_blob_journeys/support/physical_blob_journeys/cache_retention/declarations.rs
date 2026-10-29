pub(crate) use {std::num::NonZeroU64};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest};
pub(crate) use {worth_store_blob_chunks::BlobChunkSize};
pub(crate) use {worth_store_physical_format::PersistedRecordIdentity};

pub(crate) use {worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization};

pub(crate) const CHUNK: usize = 64 * 1024;

pub(crate) fn selected_record_id(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    persisted: PersistedRecordIdentity,
) -> worth_store::physical_runtime::PhysicalRecordId {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            worth_store::physical_runtime::RecordScanRequest::from_start()
                .with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(16).unwrap())
                .with_payload_limit(worth_store::physical_runtime::RecordByteLimit::new(1).unwrap()),
        )
        .unwrap();
    let mut scratch = [0_u8; 256];
    match scan.read_next_into(&mut scratch).unwrap() {
        worth_store::physical_runtime::RecordScanOutcome::Batch(batch) => batch
            .records()
            .iter()
            .find(|record| {
                record.record_id().allocation_epoch() == persisted.allocation_epoch()
                    && record.record_id().ordinal() == persisted.ordinal()
            })
            .expect("selected ordinary record")
            .record_id(),
        worth_store::physical_runtime::RecordScanOutcome::Completed(_) => panic!("ordinary record vanished from selected root"),
    }
}

pub(crate) fn read_ordinary(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    record: worth_store::physical_runtime::PhysicalRecordId,
) -> Vec<u8> {
    read_ordinary_observed(serving, record).0
}

pub(crate) fn read_ordinary_observed(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    record: worth_store::physical_runtime::PhysicalRecordId,
) -> (Vec<u8>, u64) {
    let mut read = serving
        .records()
        .unwrap()
        .open(
            record,
            worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(CHUNK as u32).unwrap()),
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