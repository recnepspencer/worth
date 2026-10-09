//! Compare the selected A successor's real controls with the checkpoint fold.

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    RecordByteLimit, RecordCountLimit, RecordScanOutcome, RecordScanRequest, ServingPhysicalRuntime,
};
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimSourceBasisV1, BlobRecordV1, PersistedRecordIdentity,
    ReleaseCheckpointAccumulatorV2, ReleaseCheckpointBatchV1, ReleaseCustodyHeadEntryV1,
    ReleasedDropCumulativeEvidenceV1,
};

pub(super) fn assert_selected_successor_controls(
    serving: &ServingPhysicalRuntime,
    head: ReleaseCustodyHeadEntryV1,
    batch: ReleaseCheckpointBatchV1,
    prior: ReleaseCheckpointAccumulatorV2,
) {
    let descriptor_bytes = selected_blob_control(serving, batch.descriptor_record());
    let BlobRecordV1::ReclaimDescriptorV3(descriptor) =
        decode_blob_record(&descriptor_bytes).unwrap()
    else {
        panic!("A's selected descriptor must be V3")
    };
    assert_eq!(
        <[u8; 32]>::from(Sha256::digest(&descriptor_bytes)),
        batch.descriptor_frame_sha256(),
    );
    assert_eq!(head.descriptor_record(), batch.descriptor_record());
    assert_eq!(
        head.descriptor_frame_sha256(),
        batch.descriptor_frame_sha256()
    );
    assert_eq!(descriptor.custody_digest(), batch.custody_digest());
    assert_eq!(descriptor.base().predecessor(), batch.predecessor());
    assert_eq!(head.predecessor(), descriptor.base().predecessor());
    assert_eq!(head.terminal(), descriptor.base().terminal());
    assert_eq!(head.manifest_record(), descriptor.base().manifest_record());
    assert_eq!(
        head.manifest_frame_sha256(),
        descriptor.base().manifest_frame_sha256(),
    );
    let manifest_bytes = selected_blob_control(serving, descriptor.base().manifest_record());
    let BlobRecordV1::DropSetManifestV3(manifest) = decode_blob_record(&manifest_bytes).unwrap()
    else {
        panic!("A's selected manifest must be V3")
    };
    assert_eq!(manifest.count(), 1);
    assert_eq!(manifest.count(), descriptor.base().manifest_count());
    assert_eq!(
        <[u8; 32]>::from(Sha256::digest(&manifest_bytes)),
        descriptor.base().manifest_frame_sha256(),
    );
    assert_eq!(head.reservation_record(), batch.reservation_record());
    assert_eq!(
        head.reservation_frame_sha256(),
        batch.reservation_frame_sha256(),
    );
    let reservation_bytes = selected_blob_control(serving, head.reservation_record());
    let BlobRecordV1::OriginalDropReserved(reservation) =
        decode_blob_record(&reservation_bytes).unwrap()
    else {
        panic!("A's selected reservation must be V1")
    };
    assert_eq!(
        <[u8; 32]>::from(Sha256::digest(&reservation_bytes)),
        head.reservation_frame_sha256(),
    );
    assert_eq!(reservation.manifest_record(), head.manifest_record());
    assert_eq!(
        reservation.manifest_frame_sha256(),
        head.manifest_frame_sha256(),
    );
    assert_eq!(
        reservation.source_basis_digest(),
        head.source_basis_digest()
    );
    let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = manifest.source_basis() else {
        panic!("A's manifest must name the released source")
    };
    assert_eq!(
        (source.object(), source.generation()),
        (head.key().object(), head.key().generation())
    );
    let evidence = ReleasedDropCumulativeEvidenceV1::new(
        batch.descriptor_record(),
        batch.descriptor_frame_sha256(),
        descriptor.custody_digest(),
        batch.reservation_record(),
        batch.reservation_frame_sha256(),
        batch.fate(),
        batch.candidate_root_generation(),
        batch.candidate_root_sha256(),
        batch.predecessor(),
        manifest.count(),
        descriptor.base().terminal(),
    )
    .unwrap();
    let (count, digest) = evidence
        .advance(
            prior.base().cumulative_dropped(),
            prior.base().cumulative_digest(),
        )
        .unwrap();
    assert_eq!(
        (batch.cumulative_dropped(), batch.cumulative_digest()),
        (count, digest)
    );
}

fn selected_blob_control(
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
                return batch
                    .payload(index)
                    .expect("selected V3 control payload")
                    .to_vec();
            }
        }
        if batch.is_complete() {
            break;
        }
    }
    panic!("selected A successor control is missing")
}
