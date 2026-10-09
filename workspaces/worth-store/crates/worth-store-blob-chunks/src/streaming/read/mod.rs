//! Read proof grammar: admit_read → observe_chunk_window → finish_verified_read.
mod classification;
mod counters;
mod denial;
mod observation;
mod orchestration;
mod receipt_construction;
mod request;
mod transitions;
mod types;
mod verification;

#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;

pub use counters::BlobStreamingReadCounterSnapshot;
pub use denial::{reject_full_blob_vec_as_streaming_read, BlobStreamingReadDenial};
pub use observation::{BlobStreamingReadObservation, BlobStreamingReadObservedChunk};
pub use receipt_construction::BlobStreamingReadCounterBackedPerformanceReceipt;
pub use request::{BlobStreamingReadRequest, BlobStreamingReadWindow};
pub use types::BlobStreamingVerifiedRead;
