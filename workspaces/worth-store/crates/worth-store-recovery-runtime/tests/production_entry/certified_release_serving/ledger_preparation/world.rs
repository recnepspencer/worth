//! One genuine publication, then the maximum checkpoint-admissible series of
//! bounded partial V3 drops. This grows selected Batch custody without a
//! many-object identity scan or an artificial control roster.

use super::*;

use worth_proof::AdmittedBlobReleaseProof;
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::MAX_CHECKPOINT_CERTIFICATE_RECORDS;

const CHUNK_BYTES: usize = 64 << 10;
const RELEASES: usize = MAX_CHECKPOINT_CERTIFICATE_RECORDS as usize - 1;
const CHUNKS: usize = RELEASES + 1;
// BlobTreeBuilder's 4,096-entry fanout writes one leaf/root for 64 chunks.
const TREE_NODE_RECORDS: usize = 1;
// The selected scan includes each payload, one publication, and at most four
// control records per completed V3 batch; 16 covers fixed ingress metadata.
const SELECTED_RECORD_LIMIT: u64 = (CHUNKS + 4 * RELEASES + 16) as u64;

pub(super) fn checkpointed_partial_release_world() -> PhysicalResidencyStoreWorld {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery_with_manifest_capacity(
        "recovered-ledger-partial-release",
        4,
    )
    .expect("initialize genuine compact-routing release world");
    let scope = admitted_blob_scope("c11.recovery.ledger.partial-release");
    let blobs = world.serving().blobs().expect("blob owner");
    let read = BlobReadLimits::new(NonZeroU64::new(SELECTED_RECORD_LIMIT).unwrap());
    let object = blobs.issue_object_id(read).expect("one object identity");
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        (CHUNKS * CHUNK_BYTES) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, world.placement(), CHUNK_BYTES as u64, read)
        .expect("begin genuine multi-chunk ingest");
    let mut payload = [0_u8; CHUNK_BYTES];
    for chunk in 0..CHUNKS {
        payload[0] = u8::try_from(chunk).expect("bounded chunk ordinal");
        ingest
            .push(&payload)
            .expect("genuine distinct payload chunk");
    }
    let published = match ingest.finish() {
        Ok(published) | Err(BlobIngestFailure::PublishedIndexPending { published, .. }) => {
            published
        }
        Err(failure) => panic!("multi-chunk publication failed: {failure:?}"),
    };
    drop(blobs);
    let marker = world
        .serving()
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("selected publication for partially released object");
    assert_eq!(published.object(), object);
    assert_eq!(published.generation().sequence(), 1);
    let publication_record = marker.record();

    for ordinal in 0..RELEASES {
        let proof = AdmittedBlobReleaseProof::certification_admit(
            world.serving().store_identity().bytes(),
            object.bytes(),
            published.generation().sequence(),
            publication_record.allocation_epoch(),
            publication_record.ordinal(),
            marker.encoded_digest(),
            [0x71; 32],
        )
        .expect("owner-issued selected publication proof");
        let request = BlobReclaimRequest::released(
            proof,
            world.placement(),
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            BlobReclaimLimits::new(
                NonZeroU64::new(SELECTED_RECORD_LIMIT).unwrap(),
                NonZeroU64::new(32 << 20).unwrap(),
                NonZeroU16::new(1).unwrap(),
            )
            .unwrap(),
        );
        let receipt = world
            .serving()
            .blobs()
            .unwrap()
            .reclaim(request)
            .unwrap_or_else(|failure| panic!("partial V3 admission {ordinal}: {failure:?}"))
            .wait()
            .unwrap_or_else(|failure| panic!("partial V3 completion {ordinal}: {failure:?}"));
        assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
        assert_eq!(receipt.dropped_records().len(), 1, "partial V3 {ordinal}");
        if ordinal == 0 {
            assert_eq!(
                receipt.dropped_records(),
                &[publication_record],
                "the first post-order drop retires the selected publication"
            );
        } else {
            assert_ne!(receipt.dropped_records(), &[publication_record]);
        }
        // The publication is dropped first but is not counted among the
        // reachable chunks and tree nodes in `remaining_payload_records`.
        assert_eq!(
            receipt.remaining_payload_records(),
            (CHUNKS + TREE_NODE_RECORDS - ordinal) as u64,
            "partial V3 {ordinal} preserves the next live chunk"
        );
    }
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0xb7; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(120_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("63-Batch checkpoint must admit")
    };
    assert!(
        matches!(handle.wait(), PhysicalCheckpointOutcome::Completed(_)),
        "63-Batch plus Accumulator checkpoint must complete"
    );
    world
}
