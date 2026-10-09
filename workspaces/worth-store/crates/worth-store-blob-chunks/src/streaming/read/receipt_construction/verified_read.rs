use super::super::types::BlobStreamingVerifiedRead;
use super::performance::counter_backed_streaming_read_performance_receipt;
use crate::{
    BlobStreamingReadCounterSnapshot, BlobStreamingReadDenial, BlobStreamingReadRequest,
    LogicalContentDigest,
};

pub(crate) fn assemble_verified_read(
    request: &BlobStreamingReadRequest,
    digest: LogicalContentDigest,
    counters: BlobStreamingReadCounterSnapshot,
) -> Result<BlobStreamingVerifiedRead, BlobStreamingReadDenial> {
    let performance = counter_backed_streaming_read_performance_receipt(counters);
    Ok(BlobStreamingVerifiedRead {
        object_id: request.object_id().clone(),
        generation: request.generation(),
        chunk_tree_root: request.chunk_tree_root().clone(),
        logical_content_digest: digest,
        counters,
        performance,
    })
}
