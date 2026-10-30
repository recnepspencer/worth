//! Read-side selected checkpoint and blob control observations for the journey.

pub(crate) use {std::fs, std::path::Path};

pub(crate) use {
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordCountLimit,
    worth_store::physical_runtime::RecordScanOutcome,
    worth_store::physical_runtime::RecordScanRequest,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_physical_format::decode_blob_record,
    worth_store_physical_format::BlobReclaimSourceBasisV1,
    worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::PersistedRecordIdentity,
    worth_store_physical_format::ReleaseCheckpointAccumulatorV2,
    worth_store_physical_format::ReleaseCheckpointBatchV1,
    worth_store_physical_format::ReleasedGenerationReclaimBasisV1,
};

pub(crate) fn selected_certificates(
    root: &Path,
) -> (
    Vec<ReleaseCheckpointBatchV1>,
    ReleaseCheckpointAccumulatorV2,
) {
    let bytes = std::fs::read(root.join("families/checkpoint.current")).unwrap();
    worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates_from_bytes(&bytes)
}

pub(crate) fn selected_blob_control(
    serving: &ServingPhysicalRuntime,
    target: PersistedRecordIdentity,
) -> Vec<u8> {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            worth_store::physical_runtime::RecordScanRequest::from_start()
                .with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(8).unwrap())
                .with_payload_limit(
                    worth_store::physical_runtime::RecordByteLimit::new(512 << 10).unwrap(),
                ),
        )
        .unwrap();
    let mut scratch = vec![0_u8; 512 << 10];
    while let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) =
        scan.read_next_into(&mut scratch).unwrap()
    {
        for index in 0..batch.records().len() {
            let record = batch.records()[index].record_id();
            if record.allocation_epoch() == target.allocation_epoch()
                && record.ordinal() == target.ordinal()
            {
                let bytes = batch.payload(index).expect("selected V3 control payload");
                return bytes.to_vec();
            }
        }
        if batch.is_complete() {
            break;
        }
    }
    panic!("C3 selected V3 control is missing")
}

pub(crate) fn selected_released_source(
    serving: &ServingPhysicalRuntime,
) -> ReleasedGenerationReclaimBasisV1 {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            worth_store::physical_runtime::RecordScanRequest::from_start()
                .with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(8).unwrap())
                .with_payload_limit(
                    worth_store::physical_runtime::RecordByteLimit::new(4096).unwrap(),
                ),
        )
        .unwrap();
    let mut scratch = [0_u8; 8192];
    let mut sources = Vec::new();
    while let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) =
        scan.read_next_into(&mut scratch).unwrap()
    {
        for index in 0..batch.records().len() {
            let Some(bytes) = batch.payload(index) else {
                continue;
            };
            if let Ok(worth_store_physical_format::BlobRecordV1::DropSetManifestV3(value)) =
                decode_blob_record(bytes)
                && let worth_store_physical_format::BlobReclaimSourceBasisV1::ReleasedGeneration(
                    source,
                ) = value.source_basis()
            {
                sources.push(source);
            }
        }
        if batch.is_complete() {
            break;
        }
    }
    let [source] = sources.as_slice() else {
        panic!("C2 retains exactly one genuine same-object V3 source")
    };
    *source
}
