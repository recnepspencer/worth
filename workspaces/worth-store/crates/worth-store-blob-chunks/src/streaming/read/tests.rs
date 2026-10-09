use worth_store_budgets::CounterEvidenceStrength;

use worth_store_security::StoreTenantScope;

use crate::publication::test_support::publish_generation_with_bytes_and_chunk_size;
use crate::test_support::{
    admitted_multichunk_sequence_for_scope, blob_scope, physical_payload_for_bytes,
};
use crate::{
    reject_full_blob_vec_as_streaming_read, BlobChunkByteRange, BlobChunkOrdinal,
    BlobCorruptionReferenceEdge, BlobCorruptionReferenceEdges, BlobQuarantineAuthority,
    BlobStreamingContentFrontier, BlobStreamingReadDenial, BlobStreamingReadObservation,
    BlobStreamingReadObservedChunk, BlobStreamingReadRequest, BlobStreamingReadWindow,
    BlobStreamingVerifiedRead,
};

const READ_CASE: &str = "phase10.streaming.read";
const READ_BYTES: &[u8] = b"abcdefghijkl";

#[test]
fn streaming_read_verification_is_independent_of_read_buffer_size() {
    let small = verify_with_window(4).expect("small read window should verify");
    let large = verify_with_window(8).expect("larger read window should verify");

    assert_eq!(small.chunk_tree_root(), large.chunk_tree_root());
    assert_eq!(
        small.logical_content_digest(),
        large.logical_content_digest()
    );
    assert_eq!(small.counters().bytes_read(), 12);
    assert_eq!(large.counters().bytes_read(), 12);
    assert_eq!(small.counters().chunks_read(), 3);
    assert_eq!(large.counters().chunks_read(), 3);
    assert_eq!(small.counters().chunks_verified(), 3);
    assert_eq!(large.counters().chunks_verified(), 3);
    assert_eq!(small.counters().chunk_checksum_verifications(), 3);
    assert_eq!(large.counters().chunk_checksum_verifications(), 3);
    assert_eq!(
        small.counters().counter_strength(),
        CounterEvidenceStrength::Exact
    );
    assert!(small
        .counter_backed_performance_receipt()
        .counter_rows()
        .iter()
        .any(|row| row.name().as_str().ends_with(".chunks_read") && row.observed_count() == 3));
}

#[test]
fn missing_reordered_corrupt_cold_and_whole_expected_paths_deny() {
    let missing = verify_observations(observations_for(b"abcdefghijkl", 4).into_iter().take(2))
        .expect_err("missing tail chunk must deny");
    assert!(matches!(
        missing,
        BlobStreamingReadDenial::MissingChunk { .. }
    ));

    let mut reordered = observations_for(b"abcdefghijkl", 4);
    reordered.swap(0, 1);
    let denial = verify_observations(reordered).expect_err("reordered chunks must deny");
    assert!(matches!(
        denial,
        BlobStreamingReadDenial::ReorderedChunk { .. }
    ));

    let corrupt = observations_for(b"abcdZZZZijkl", 4);
    let denial = verify_observations(corrupt).expect_err("corrupted chunk must deny");
    assert!(matches!(
        denial,
        BlobStreamingReadDenial::CorruptedChunk {
            damage_case,
            diagnostics,
            ..
        } if damage_case == crate::BlobDamageCase::ChecksumMismatch
            && diagnostics.quarantine().counters().quarantine_holds() == 1
            && diagnostics.quarantine().counters().read_detections() == 1
    ));

    let cold = [
        observations_for(b"abcdefghijkl", 4).remove(0),
        BlobStreamingReadObservation::cold_unavailable(
            BlobChunkOrdinal::first().next(),
            BlobChunkByteRange::new(4, 4).unwrap(),
        ),
    ];
    let denial = verify_observations(cold).expect_err("cold-unavailable chunk must deny");
    assert!(matches!(
        denial,
        BlobStreamingReadDenial::ColdChunkUnavailable { .. }
    ));

    assert_eq!(
        reject_full_blob_vec_as_streaming_read(b"abcdefghijkl".to_vec()),
        BlobStreamingReadDenial::WholeObjectExpectedBufferRejected { bytes: 12 }
    );
}

#[test]
fn streaming_read_request_denies_unrelated_corruption_reference_edges() {
    let (published, visible) =
        publish_generation_with_bytes_and_chunk_size(READ_CASE, READ_BYTES, 4);
    let (unrelated_published, _) =
        publish_generation_with_bytes_and_chunk_size("phase10.streaming.unrelated", READ_BYTES, 4);
    let reference_edges = BlobCorruptionReferenceEdges::from_admitted_edges(&[
        BlobCorruptionReferenceEdge::from_reachability_staging_identity(
            published.staging_identity(),
        ),
        BlobCorruptionReferenceEdge::from_reachability_staging_identity(
            unrelated_published.staging_identity(),
        ),
    ])
    .expect("distinct edge witnesses should construct before request binding");

    let denied =
        BlobStreamingReadRequest::from_published_generation(visible, frontier(), reference_edges)
            .expect_err("unrelated affected edge must deny before read publication");

    assert!(matches!(
        denied,
        BlobStreamingReadDenial::CorruptionReferenceEdgeMismatch(_)
    ));
}

fn verify_with_window(
    window_bytes: u64,
) -> Result<BlobStreamingVerifiedRead, BlobStreamingReadDenial> {
    verify_observations_with_window(
        window_bytes,
        observations_for(b"abcdefghijkl", window_bytes),
    )
}

fn verify_observations(
    observations: impl IntoIterator<Item = BlobStreamingReadObservation>,
) -> Result<BlobStreamingVerifiedRead, BlobStreamingReadDenial> {
    verify_observations_with_window(4, observations)
}

fn verify_observations_with_window(
    window_bytes: u64,
    observations: impl IntoIterator<Item = BlobStreamingReadObservation>,
) -> Result<BlobStreamingVerifiedRead, BlobStreamingReadDenial> {
    BlobStreamingVerifiedRead::verify_bounded_content(
        request(),
        BlobStreamingReadWindow::bounded(window_bytes)?,
        BlobQuarantineAuthority::from_current_store_authority(
            crate::lifecycle::generation_registry_test_support::current_authority(
                "phase10.streaming.read.quarantine",
                "quarantine",
            ),
        ),
        CounterEvidenceStrength::Exact,
        observations,
    )
}

fn request() -> BlobStreamingReadRequest {
    let (published, visible) =
        publish_generation_with_bytes_and_chunk_size(READ_CASE, READ_BYTES, 4);
    let reference_edges = BlobCorruptionReferenceEdges::from_reachability_staging_identity(
        published.staging_identity(),
    )
    .expect("published reachability staging identity should provide corruption reference edge");
    BlobStreamingReadRequest::from_published_generation(visible, frontier(), reference_edges)
        .expect("published generation should bind streaming read request")
}

fn frontier() -> BlobStreamingContentFrontier {
    let sequence = admitted_multichunk_sequence_for_scope(
        blob_scope(READ_CASE, StoreTenantScope::TenantPhysicalBoundary),
        READ_BYTES,
        4,
    );
    BlobStreamingContentFrontier::from_sequence(&sequence)
}

fn observations_for(bytes: &[u8], window_bytes: u64) -> Vec<BlobStreamingReadObservation> {
    bytes
        .chunks(4)
        .enumerate()
        .map(|(index, chunk)| {
            let ordinal = ordinal(index as u64);
            let observed = BlobStreamingReadObservedChunk::from_store_payload(
                ordinal,
                index as u64 * 4,
                physical_payload_for_bytes(chunk),
                BlobStreamingReadWindow::bounded(window_bytes).unwrap(),
            )
            .expect("bounded read chunk should admit");
            BlobStreamingReadObservation::from_chunk(observed)
        })
        .collect()
}

fn ordinal(value: u64) -> BlobChunkOrdinal {
    let mut ordinal = BlobChunkOrdinal::first();
    for _ in 0..value {
        ordinal = ordinal.next();
    }
    ordinal
}
