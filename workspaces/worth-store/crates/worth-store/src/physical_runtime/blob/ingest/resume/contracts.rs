use std::num::NonZeroU64;

use worth_store_physical_format::BlobRecordDenial;

use super::super::{BlobIngestClaimDenial, BlobIngestFailure};
use crate::physical_runtime::{RecordReadError, RecordScanError, RecordStreamFailure};

/// Explicit bounds for cold, selected-root reconstruction. Metadata capacity
/// exhaustion denies readmission; it never abandons durable session custody.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobResumeLimits {
    maximum_scanned_records: NonZeroU64,
    metadata_bytes: NonZeroU64,
}

impl BlobResumeLimits {
    pub const fn new(maximum_scanned_records: NonZeroU64, metadata_bytes: NonZeroU64) -> Self {
        Self {
            maximum_scanned_records,
            metadata_bytes,
        }
    }

    pub const fn maximum_scanned_records(self) -> NonZeroU64 {
        self.maximum_scanned_records
    }
    pub const fn metadata_bytes(self) -> NonZeroU64 {
        self.metadata_bytes
    }
}

#[derive(Debug)]
pub enum BlobResumeFailure {
    Claim(BlobIngestClaimDenial),
    Ingest(BlobIngestFailure),
    Format(BlobRecordDenial),
    Scan(RecordScanError),
    Read(RecordReadError),
    Stream(RecordStreamFailure),
    Reuse(crate::physical_runtime::blob::BlobDedupeFailure),
    ForeignStore,
    DeclarationMismatch,
    ScopeMismatch,
    AlreadyPublished,
    /// Store released the generation this session published. The session
    /// never resumes: finishing it would publish its identity a second time.
    AlreadyReleased,
    AlreadyAbandoned,
    Expired,
    ScanBoundExhausted,
    MetadataCapacity,
    ScratchUnavailable,
    ConflictingClaims,
    TreeConflict,
}

/// Reconstruction work is separate from ordinary streaming-ingest cost.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BlobResumeObservation {
    pub(super) scanned_records: u64,
    pub(super) selected_payload_bytes: u64,
    pub(super) rehashed_chunks: u64,
    pub(super) rehashed_bytes: u64,
    pub(super) reused_nodes: u64,
    pub(super) metadata_capacity_bytes: u64,
}

impl BlobResumeObservation {
    pub const fn scanned_records(self) -> u64 {
        self.scanned_records
    }
    pub const fn selected_payload_bytes(self) -> u64 {
        self.selected_payload_bytes
    }
    pub const fn rehashed_chunks(self) -> u64 {
        self.rehashed_chunks
    }
    pub const fn rehashed_bytes(self) -> u64 {
        self.rehashed_bytes
    }
    pub const fn reused_nodes(self) -> u64 {
        self.reused_nodes
    }
    pub const fn metadata_capacity_bytes(self) -> u64 {
        self.metadata_capacity_bytes
    }
}
