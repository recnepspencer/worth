use std::num::{NonZeroU16, NonZeroU64};

use worth_proof::AdmittedBlobReleaseProof;
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedBlobScope, AdmittedPhysicalRecordFormat, BlobCheckpointLimit, BlobIngestDeclaration,
    BlobReadLimits, BlobReclaimDeferral, BlobReclaimDisposition, BlobReclaimFailure,
    BlobReclaimLimits, BlobReclaimRequest, ManifestEntryCapacity, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
    PhysicalMutationDeadline, PhysicalPageSizeClass, PhysicalRecordFormatDeclaration,
    PhysicalRecordPlacementPolicy, PublishedBlobGeneration, ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;

use super::fixture::{admitted_blob_scope, placement, serving_from_initialization};

const CHUNK: usize = 64 << 10;
const SHARED_CHUNK: usize = 256 << 10;

#[path = "blob_reclaim_released/control_placement.rs"]
mod control_placement;
#[path = "blob_reclaim_released/shared_reuse.rs"]
mod shared_reuse;

#[test]
fn released_generation_one_record_batches_keep_post_order_custody_and_end_in_no_effect() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.release.post_order.scope");
    let blobs = serving.blobs().unwrap();
    let read_limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK as u64, read_limits)
        .unwrap();
    ingest.push(&vec![0x31; CHUNK]).unwrap();
    ingest.push(&vec![0x52; CHUNK]).unwrap();
    let abandoned_token = ingest.resume_token();
    let published = ingest.finish().unwrap();
    drop(blobs);

    let marker = serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("published generation has selected C5 marker");
    let release_limits = || {
        BlobReclaimLimits::new(
            NonZeroU64::new(256).unwrap(),
            NonZeroU64::new(64 << 20).unwrap(),
            NonZeroU16::new(1).unwrap(),
        )
        .unwrap()
    };
    let proof = |store, frame_sha256| {
        let record = marker.record();
        AdmittedBlobReleaseProof::certification_admit(
            store,
            object.bytes(),
            published.generation().sequence(),
            record.allocation_epoch(),
            record.ordinal(),
            frame_sha256,
            [0x71; 32],
        )
        .unwrap()
    };
    let request = || {
        BlobReclaimRequest::released(
            proof(serving.store_identity().bytes(), marker.encoded_digest()),
            placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            release_limits(),
        )
    };

    let before_denials = super::blob_frontier::selected_blob_records(&serving);
    let held = serving.records().unwrap();
    assert!(matches!(
        serving.blobs().unwrap().reclaim(request()),
        Err(BlobReclaimFailure::Deferred(
            BlobReclaimDeferral::ProtectedReader
        ))
    ));
    drop(held);
    assert!(matches!(
        serving
            .blobs()
            .unwrap()
            .reclaim(BlobReclaimRequest::abandoned(
                abandoned_token,
                &scope,
                placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                release_limits(),
            )),
        Err(BlobReclaimFailure::AlreadyPublished)
    ));
    assert!(matches!(
        serving
            .blobs()
            .unwrap()
            .reclaim(BlobReclaimRequest::released(
                proof(serving.store_identity().bytes(), [0x91; 32]),
                placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                release_limits(),
            )),
        Err(BlobReclaimFailure::DeclarationMismatch)
    ));
    assert!(matches!(
        serving
            .blobs()
            .unwrap()
            .reclaim(BlobReclaimRequest::released(
                proof([0x92; 16], marker.encoded_digest()),
                placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                release_limits(),
            )),
        Err(BlobReclaimFailure::ForeignStore)
    ));
    assert_eq!(
        super::blob_frontier::selected_blob_records(&serving),
        before_denials,
        "all negative admissions must leave the selected root unchanged"
    );

    let first = serving
        .blobs()
        .unwrap()
        .reclaim(request())
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(first.disposition(), BlobReclaimDisposition::Dropped);
    assert_eq!(first.dropped_records(), &[marker.record()]);
    assert!(first.remaining_payload_records() > 0);
    assert!(
        !super::blob_frontier::selected_blob_records(&serving)
            .iter()
            .any(|(record, _)| {
                record.allocation_epoch() == marker.record().allocation_epoch()
                    && record.ordinal() == marker.record().ordinal()
            }),
        "completed first batch must remove publication from the selected C5 root"
    );
    let mut remaining = first.remaining_payload_records();
    for _ in 0..8 {
        let receipt = serving
            .blobs()
            .unwrap()
            .reclaim(request())
            .unwrap()
            .wait()
            .unwrap();
        assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
        assert_eq!(receipt.dropped_records().len(), 1);
        assert!(receipt.remaining_payload_records() < remaining);
        remaining = receipt.remaining_payload_records();
        if remaining == 0 {
            break;
        }
    }
    assert_eq!(
        remaining, 0,
        "all exclusive publication-tree custody was dropped"
    );
    let repeated = serving
        .blobs()
        .unwrap()
        .reclaim(request())
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(
        repeated.disposition(),
        BlobReclaimDisposition::ProvenNoEffect
    );
    assert!(repeated.dropped_records().is_empty());
    serving.close();
}

#[test]
fn shared_source_release_requires_c8_before_reopen() {
    shared_reuse::source_first();
}

#[test]
fn cancelled_pre_effect_release_restores_checkpoint_and_reclaim_admission() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.release.cancel.scope");
    let blobs = serving.blobs().unwrap();
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK as u64, limits)
        .unwrap();
    ingest.push(&vec![0x31; CHUNK]).unwrap();
    ingest.push(&vec![0x52; CHUNK]).unwrap();
    let published = ingest.finish().unwrap();
    drop(blobs);
    let marker = serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .unwrap();
    let request = || {
        let proof = AdmittedBlobReleaseProof::certification_admit(
            serving.store_identity().bytes(),
            object.bytes(),
            published.generation().sequence(),
            marker.record().allocation_epoch(),
            marker.record().ordinal(),
            marker.encoded_digest(),
            [0x72; 32],
        )
        .unwrap();
        BlobReclaimRequest::released(
            proof,
            placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            BlobReclaimLimits::new(
                NonZeroU64::new(256).unwrap(),
                NonZeroU64::new(64 << 20).unwrap(),
                NonZeroU16::new(16).unwrap(),
            )
            .unwrap(),
        )
    };
    let handle = serving.blobs().unwrap().reclaim(request()).unwrap();
    drop(handle);
    let checkpoint = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x73; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(checkpoint).into_raw()
    else {
        panic!("pre-effect cancellation must restore checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    let retry = serving
        .blobs()
        .unwrap()
        .reclaim(request())
        .expect("pre-effect cancellation must restore reclaim admission");
    drop(retry);
    serving.close();
}

fn publish_one(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
    payload: &[u8],
) -> PublishedBlobGeneration {
    let blobs = serving.blobs().unwrap();
    let limits = BlobReadLimits::new(NonZeroU64::new(256).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(SHARED_CHUNK as u64).unwrap(),
        payload.len() as u64,
        scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(
            declaration,
            shared_placement(),
            (SHARED_CHUNK / 2) as u64,
            limits,
        )
        .unwrap();
    for piece in payload.chunks(SHARED_CHUNK / 2) {
        ingest.push(piece).unwrap();
    }
    ingest.finish().unwrap()
}

fn shared_placement() -> worth_store::physical_runtime::AdmittedRecordPlacementPolicy {
    let format = shared_format();
    PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(512).unwrap())
        .admit(format)
        .unwrap()
}

fn shared_format() -> AdmittedPhysicalRecordFormat {
    AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder()
            .page_size(PhysicalPageSizeClass::KiB64)
            .admit()
            .unwrap(),
    )
}
