//! Read-side selected checkpoint and blob control observations for the journey.

use std::{fs, path::Path};

use worth_store::physical_runtime::{
    RecordByteLimit, RecordCountLimit, RecordScanOutcome, RecordScanRequest, ServingPhysicalRuntime,
};
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimSourceBasisV1, BlobRecordV1, PersistedRecordIdentity,
    ReleaseCheckpointAccumulatorV2, ReleaseCheckpointBatchV1, ReleasedGenerationReclaimBasisV1,
};

pub(super) fn selected_certificates(
    root: &Path,
) -> (
    Vec<ReleaseCheckpointBatchV1>,
    ReleaseCheckpointAccumulatorV2,
) {
    let bytes = fs::read(root.join("families/checkpoint.current")).unwrap();
    super::super::release_reopen::selected_release_certificates_from_bytes(&bytes)
}

pub(super) fn selected_blob_control(
    serving: &ServingPhysicalRuntime,
    target: PersistedRecordIdentity,
) -> Vec<u8> {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(8).unwrap())
                .with_payload_limit(RecordByteLimit::new(512 << 10).unwrap()),
        )
        .unwrap();
    let mut scratch = vec![0_u8; 512 << 10];
    while let RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
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

pub(super) fn selected_released_source(
    serving: &ServingPhysicalRuntime,
) -> ReleasedGenerationReclaimBasisV1 {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(8).unwrap())
                .with_payload_limit(RecordByteLimit::new(4096).unwrap()),
        )
        .unwrap();
    let mut scratch = [0_u8; 8192];
    let mut sources = Vec::new();
    while let RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        for index in 0..batch.records().len() {
            let Some(bytes) = batch.payload(index) else {
                continue;
            };
            if let Ok(BlobRecordV1::DropSetManifestV3(value)) = decode_blob_record(bytes) {
                if let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = value.source_basis() {
                    sources.push(source);
                }
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
