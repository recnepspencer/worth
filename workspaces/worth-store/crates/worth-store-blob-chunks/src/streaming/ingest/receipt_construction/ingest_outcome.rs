use super::super::frontier::BlobStreamingContentFrontier;
use super::super::types::BlobStreamingIngest;
use super::performance::counter_backed_streaming_performance_receipt;
use crate::{
    AdmittedBlobChunkSequence, BlobStreamingIngestCounterSnapshot, BlobStreamingIngestDenial,
    BlobStreamingResumePosture,
};

pub(crate) fn emit_ingest_receipt(
    sequence: AdmittedBlobChunkSequence,
    counters: BlobStreamingIngestCounterSnapshot,
) -> Result<BlobStreamingIngest, BlobStreamingIngestDenial> {
    let frontier = BlobStreamingContentFrontier::from_sequence(&sequence);
    let resumability = BlobStreamingResumePosture::from_frontier(&frontier);
    let performance = counter_backed_streaming_performance_receipt(counters);
    Ok(BlobStreamingIngest::from_bounded_parts(
        sequence,
        frontier,
        resumability,
        counters,
        performance,
    ))
}
