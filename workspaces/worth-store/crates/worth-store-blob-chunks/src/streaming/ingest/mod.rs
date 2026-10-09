//! Ingest proof grammar: admit_stream → advance_frontier → verify_chunk_window → emit_ingest_receipt.
mod admission;
mod chunk_movement;
mod classification;
mod counters;
mod denial;
mod frontier;
mod orchestration;
mod receipt_construction;
mod request;
mod source;
mod transitions;
mod types;
mod verification;

#[cfg(test)]
mod ingest_test_support;
#[cfg(test)]
mod ingest_tests;
#[cfg(test)]
mod pressure_tests;

pub use admission::{
    reject_scalar_backend_api_as_streaming_ingest, BlobStreamingPressureAdmission,
};
pub use counters::BlobStreamingIngestCounterSnapshot;
pub use denial::{reject_full_blob_vec_as_streaming_ingest, BlobStreamingIngestDenial};
pub use frontier::BlobStreamingContentFrontier;
pub use receipt_construction::BlobStreamingCounterBackedPerformanceReceipt;
pub use request::{BlobStreamingIngestRequest, BlobStreamingWindow};
pub use source::{BlobStreamingChunkWriter, BlobStreamingSourceFrame, BlobStreamingWrittenChunk};
pub use types::BlobStreamingIngest;
