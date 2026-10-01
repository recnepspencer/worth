// --- Capabilities (admission handles, next-step types) ---
pub use crate::streaming::{
    run_resumable_streaming_ingest, BlobChunkStreamingOperation, BlobChunkStreamingOperationKind,
    BlobChunkStreamingWindow, BlobStreamingChunkWriter, BlobStreamingContentFrontier,
    BlobStreamingIngest, BlobStreamingIngestRequest, BlobStreamingPressureAdmission,
    BlobStreamingReadRequest, BlobStreamingResumeAdmission,
};
// --- Outcomes (transition receipts) ---
pub use crate::streaming::{
    BlobChunkStreamingObservation, BlobChunkStreamingResidencyProof,
    BlobStreamingCounterBackedPerformanceReceipt, BlobStreamingReadCounterBackedPerformanceReceipt,
    BlobStreamingReadObservation, BlobStreamingReadObservedChunk, BlobStreamingReadWindow,
    BlobStreamingResumePosture, BlobStreamingSourceFrame, BlobStreamingVerifiedRead,
    BlobStreamingWindow, BlobStreamingWrittenChunk,
};
// --- Denials (classified failure enums) ---
pub use crate::streaming::{
    BlobChunkStreamingDenial, BlobStreamingIngestDenial, BlobStreamingReadDenial,
};
// --- Counter witnesses (read-only snapshots) ---
pub use crate::streaming::{
    BlobChunkStreamingCounterSnapshot, BlobStreamingIngestCounterSnapshot,
    BlobStreamingReadCounterSnapshot,
};
