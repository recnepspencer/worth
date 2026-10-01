use std::num::NonZeroU64;

use worth_store::physical_runtime::{
    AdmittedBlobScope, BlobCheckpointLimit, BlobDedupeFailure, BlobIngestDeclaration,
    BlobIngestFailure, BlobReadLimits, PhysicalMutationDeadline, RecordReadDenial,
    ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::{decode_blob_record, BlobRecordV1};

use super::super::{shared_placement, SHARED_CHUNK as CHUNK};

pub(super) fn assert_denied_after_source_release(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
    payload: &[u8],
) {
    let blobs = serving.blobs().unwrap();
    let limits = BlobReadLimits::new(NonZeroU64::new(256).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        payload.len() as u64,
        scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, shared_placement(), (CHUNK / 2) as u64, limits)
        .unwrap();
    let session = ingest.session_id().bytes();
    for piece in payload.chunks(CHUNK / 2) {
        match ingest.push(piece) {
            Ok(()) => {}
            Err(BlobIngestFailure::Dedupe(BlobDedupeFailure::SourceRead(error)))
                if error.denial() == RecordReadDenial::RecordNotFound =>
            {
                assert_no_reuse_claim_for(serving, session);
                return;
            }
            Err(error) => panic!("unexpected fresh ingest result after source release: {error:?}"),
        }
    }
    match ingest.finish() {
        Ok(_) | Err(BlobIngestFailure::PublishedIndexPending { .. }) => {}
        Err(error) => panic!("unexpected fresh ingest finish after source release: {error:?}"),
    }
    assert_no_reuse_claim_for(serving, session);
}

fn assert_no_reuse_claim_for(serving: &ServingPhysicalRuntime, session: [u8; 16]) {
    assert!(
        !super::super::super::blob_frontier::selected_blob_records(serving)
            .iter()
            .any(|(_, bytes)| match decode_blob_record(bytes) {
                Ok(BlobRecordV1::ChunkReuseClaim(value)) => value.destination_session() == session,
                Ok(BlobRecordV1::ChunkReuseClaimV2(value)) =>
                    value.claim().destination_session() == session,
                _ => false,
            })
    );
}
