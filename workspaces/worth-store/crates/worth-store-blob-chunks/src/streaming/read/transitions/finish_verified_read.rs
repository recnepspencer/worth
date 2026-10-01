use super::super::receipt_construction::verified_read::assemble_verified_read;
use super::super::verification::{
    frontier_coverage, logical_content_digest, StreamingReadVerifier,
};
use crate::{BlobStreamingReadCounterSnapshot, BlobStreamingReadDenial};

pub(crate) fn finish_verified_read(
    verifier: StreamingReadVerifier,
    counters: BlobStreamingReadCounterSnapshot,
) -> Result<super::super::types::BlobStreamingVerifiedRead, BlobStreamingReadDenial> {
    frontier_coverage::verify_all_leaves_consumed(
        &verifier.request,
        verifier.next_index,
        counters,
    )?;
    let frontier = verifier.request.frontier().proof_frontier();
    let digest = logical_content_digest::finalize_logical_content_digest(
        verifier.logical_content_basis,
        frontier.total_bytes(),
        frontier.chunk_count(),
    );
    if &digest != verifier.request.logical_content_digest() {
        return Err(crate::BlobStreamingReadDenial::LogicalContentDigestMismatch);
    }
    assemble_verified_read(&verifier.request, digest, counters)
}
