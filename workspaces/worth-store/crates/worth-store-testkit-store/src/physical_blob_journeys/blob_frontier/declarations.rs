pub use {std::num::NonZeroU64};

pub use {sha2::Digest, sha2::Sha256};
pub use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobResumeToken, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalRecordId, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest, worth_store::physical_runtime::ServingPhysicalRuntime};
pub use {worth_store_blob_chunks::BlobChunkSize};
pub use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1, worth_store_physical_format::BlobSessionFrontierV1, worth_store_physical_format::PersistedRecordIdentity};

pub use {crate::fixture::admitted_blob_scope, crate::fixture::placement, crate::fixture::serving_from_initialization, crate::fixture::serving_from_open};



pub const CHUNK: usize = 64 * 1024;

pub fn selected_root_generation(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .records()
        .unwrap()
        .protected_root()
        .root()
        .generation()
        .get()
}

pub fn selected_blob_records(
    serving: &ServingPhysicalRuntime,
) -> Vec<(PhysicalRecordId, Vec<u8>)> {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            worth_store::physical_runtime::RecordScanRequest::from_start()
                .with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(8).unwrap())
                .with_payload_limit(worth_store::physical_runtime::RecordByteLimit::new(236).unwrap()),
        )
        .unwrap();
    let mut scratch = [0_u8; 4096];
    let mut selected = Vec::new();
    while let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        for row in batch.records() {
            selected.push((row.record_id(), row.declared_payload_bytes()));
        }
        if batch.is_complete() {
            break;
        }
    }
    drop(scan);
    selected
        .into_iter()
        .map(|(record, length)| (record, read_selected_record(serving, record, length)))
        .collect()
}

/// Scans selected routing while copying only the small declaration/frontier
/// controls. The 64 KiB chunk frames remain deferred metadata identities.
pub fn selected_control_inventory(
    serving: &ServingPhysicalRuntime,
) -> (
    Vec<(PhysicalRecordId, Vec<u8>)>,
    Vec<(PhysicalRecordId, u64)>,
) {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            worth_store::physical_runtime::RecordScanRequest::from_start()
                .with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(8).unwrap())
                .with_payload_limit(worth_store::physical_runtime::RecordByteLimit::new(236).unwrap()),
        )
        .unwrap();
    let mut scratch = [0_u8; 4096];
    let mut controls = Vec::new();
    let mut deferred = Vec::new();
    while let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        for (index, row) in batch.records().iter().enumerate() {
            match batch.payload(index) {
                Some(bytes) => controls.push((row.record_id(), bytes.to_vec())),
                None => deferred.push((row.record_id(), row.declared_payload_bytes())),
            }
        }
        if batch.is_complete() {
            break;
        }
    }
    (controls, deferred)
}

pub fn read_selected_record(
    serving: &ServingPhysicalRuntime,
    record: PhysicalRecordId,
    length: u64,
) -> Vec<u8> {
    let reader = serving.records().unwrap();
    let mut stream = reader
        .open(
            record,
            worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(u32::try_from(length).unwrap()).unwrap()),
        )
        .unwrap();
    let mut bytes = Vec::with_capacity(length as usize);
    let mut frame = [0_u8; 8192];
    loop {
        let read = stream.read_next(&mut frame).unwrap();
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&frame[..read]);
    }
    assert_eq!(bytes.len() as u64, length);
    bytes
}

pub fn assert_record_binding(record: PhysicalRecordId, selected: PersistedRecordIdentity) {
    assert_eq!(record.allocation_epoch(), selected.allocation_epoch());
    assert_eq!(record.ordinal(), selected.ordinal());
}