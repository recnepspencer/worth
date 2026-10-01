//! The continuation child alone consumes C8 custody, performs real native
//! retirement/checkpoint, then attempts a second V3 WAL descriptor.

use std::{
    num::{NonZeroU16, NonZeroU64},
    path::Path,
};

use worth_proof::AdmittedBlobReleaseProof;
use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobIngestFailure, BlobReadLimits,
    BlobReclaimLimits, BlobReclaimRequest, PhysicalMutationDeadline, RecordByteLimit,
    RecordCountLimit, RecordScanOutcome, RecordScanRequest,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::{decode_blob_record, BlobReclaimSourceBasisV1, BlobRecordV1};

use super::park_at_descriptor_wal;

pub(super) fn child(root: &Path, marker: &Path) {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        super::super::certified_release_serving::request(root),
    );
    let handoff = match outcome {
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) => handoff,
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(block) => panic!(
            "second C8 blocked at {:?}: {:?}",
            block.kind,
            block.evidence().planning_denial,
        ),
        other => panic!("second process could not recover first pending V3: {other:?}"),
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("first C8 custody seal");
    let serving = super::super::certified_release_serving::admit_serving_with_seal(root, seal);
    let before_retirement = serving.certification_charged_growth_bytes();
    serving
        .retire_displaced_segment()
        .expect("first recovered released drop must retire its native extent under C8 seal");
    assert!(serving.certification_charged_growth_bytes() < before_retirement);
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
    drop(scan);
    let [source] = sources.as_slice() else {
        panic!("first C8 drop must retain one selected V3 source manifest")
    };
    let proof = AdmittedBlobReleaseProof::certification_admit(
        serving.store_identity().bytes(),
        source.object(),
        source.generation(),
        source.publication_record().allocation_epoch(),
        source.publication_record().ordinal(),
        source.publication_frame_sha256(),
        source.issuer_evidence_sha256(),
    )
    .unwrap();
    let request = BlobReclaimRequest::released(
        proof,
        worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
            .manifest_capacity(
                worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap(),
            )
            .admit(
                worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
                    worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
                        .admit()
                        .unwrap(),
                ),
            )
            .unwrap(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
        BlobReclaimLimits::new(
            NonZeroU64::new(128).unwrap(),
            NonZeroU64::new(32 << 20).unwrap(),
            NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    );
    park_at_descriptor_wal(&serving, request, marker, b"second-descriptor-wal");
}

pub(super) fn distinct_child(root: &Path, marker: &Path) {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        super::super::certified_release_serving::request(root),
    );
    let handoff = match outcome {
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) => handoff,
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(block) => panic!(
            "distinct release C8 blocked at {:?}: {:?}",
            block.kind,
            block.evidence().planning_denial,
        ),
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
            indeterminate,
        ) => panic!(
            "distinct release C8 publication indeterminate: handoff={:?}, reopen={:?}",
            indeterminate.handoff_failure(),
            indeterminate.reopen_failure(),
        ),
        other => panic!("distinct release requires first C8 seal: {other:?}"),
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("first C8 seal");
    let serving = super::super::certified_release_serving::admit_serving_with_seal(root, seal);
    let first_v3_result = super::selected_media::read(root);
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap())
        .admit(
            worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
                worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
                    .admit()
                    .unwrap(),
            ),
        )
        .unwrap();
    let scope = super::super::admitted_blob_scope("c11.recovery.distinct-v3.scope");
    let blobs = serving.blobs().unwrap();
    let read = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(64 << 10).unwrap(),
        128 << 10,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement, 64 << 10, read)
        .unwrap();
    ingest.push(&vec![0xa5; 64 << 10]).unwrap();
    ingest.push(&vec![0xb6; 64 << 10]).unwrap();
    let published = match ingest.finish() {
        Ok(published) | Err(BlobIngestFailure::PublishedIndexPending { published, .. }) => {
            published
        }
        Err(failure) => panic!("distinct blob publication: {failure:?}"),
    };
    drop(blobs);
    super::selected_media::assert_distinct_ingest_step(
        &first_v3_result,
        &super::selected_media::read(root),
    );
    let publication = serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .unwrap();
    assert_eq!(published.object(), object);
    assert_eq!(published.generation().sequence(), 1);
    let proof = AdmittedBlobReleaseProof::certification_admit(
        serving.store_identity().bytes(),
        object.bytes(),
        published.generation().sequence(),
        publication.record().allocation_epoch(),
        publication.record().ordinal(),
        publication.encoded_digest(),
        [0x72; 32],
    )
    .unwrap();
    let request = BlobReclaimRequest::released(
        proof,
        placement,
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
        BlobReclaimLimits::new(
            NonZeroU64::new(128).unwrap(),
            NonZeroU64::new(32 << 20).unwrap(),
            NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    );
    park_at_descriptor_wal(&serving, request, marker, b"second-descriptor-wal");
}
