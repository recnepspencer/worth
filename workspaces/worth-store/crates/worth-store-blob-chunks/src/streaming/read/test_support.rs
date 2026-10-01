use worth_store_budgets::CounterEvidenceStrength;
use worth_store_security::StoreTenantScope;

use crate::publication::test_support::publish_generation_with_bytes_and_chunk_size;
use crate::test_support::{
    admitted_multichunk_sequence_for_scope, blob_scope, physical_payload_for_bytes,
};
use crate::{
    BlobChunkOrdinal, BlobCorruptionReferenceEdges, BlobGenerationPublished,
    BlobQuarantineAuthority, BlobStreamingContentFrontier, BlobStreamingReadObservation,
    BlobStreamingReadObservedChunk, BlobStreamingReadRequest, BlobStreamingReadWindow,
    BlobStreamingVerifiedRead, BlobVisibleGeneration,
};

pub(crate) fn layout_runtime_case(
    case: &str,
    bytes: &[u8],
    chunk_size: u64,
    window_bytes: u64,
) -> (
    BlobGenerationPublished,
    BlobVisibleGeneration,
    BlobStreamingReadRequest,
    BlobStreamingVerifiedRead,
) {
    let (published, visible) =
        publish_generation_with_bytes_and_chunk_size(case, bytes, chunk_size);
    let request = request(case, bytes, chunk_size, visible.clone(), &published);
    let verified = BlobStreamingVerifiedRead::verify_bounded_content(
        request.clone(),
        BlobStreamingReadWindow::bounded(window_bytes).unwrap(),
        quarantine_authority(case),
        CounterEvidenceStrength::Exact,
        observations_for(bytes, chunk_size, window_bytes),
    )
    .expect("streaming runtime case should verify through bounded production path");
    (published, visible, request, verified)
}

fn request(
    case: &str,
    bytes: &[u8],
    chunk_size: u64,
    visible: BlobVisibleGeneration,
    published: &BlobGenerationPublished,
) -> BlobStreamingReadRequest {
    let reference_edges = BlobCorruptionReferenceEdges::from_reachability_staging_identity(
        published.staging_identity(),
    )
    .expect("published generation should supply corruption reference edge");
    BlobStreamingReadRequest::from_published_generation(
        visible,
        frontier(case, bytes, chunk_size),
        reference_edges,
    )
    .expect("published generation should bind streaming read request")
}

fn frontier(case: &str, bytes: &[u8], chunk_size: u64) -> BlobStreamingContentFrontier {
    let sequence = admitted_multichunk_sequence_for_scope(
        blob_scope(case, StoreTenantScope::TenantPhysicalBoundary),
        bytes,
        chunk_size,
    );
    BlobStreamingContentFrontier::from_sequence(&sequence)
}

fn observations_for(
    bytes: &[u8],
    chunk_size: u64,
    window_bytes: u64,
) -> Vec<BlobStreamingReadObservation> {
    bytes
        .chunks(chunk_size as usize)
        .enumerate()
        .map(|(index, chunk)| {
            let observed = BlobStreamingReadObservedChunk::from_store_payload(
                ordinal(index as u64),
                index as u64 * chunk_size,
                physical_payload_for_bytes(chunk),
                BlobStreamingReadWindow::bounded(window_bytes).unwrap(),
            )
            .expect("bounded read chunk should admit");
            BlobStreamingReadObservation::from_chunk(observed)
        })
        .collect()
}

fn quarantine_authority(case: &str) -> BlobQuarantineAuthority {
    BlobQuarantineAuthority::from_current_store_authority(
        crate::lifecycle::generation_registry_test_support::current_authority(
            &format!("{case}.quarantine"),
            "quarantine",
        ),
    )
}

fn ordinal(value: u64) -> BlobChunkOrdinal {
    let mut ordinal = BlobChunkOrdinal::first();
    for _ in 0..value {
        ordinal = ordinal.next();
    }
    ordinal
}
