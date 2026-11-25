#![forbid(unsafe_code)]

//! Self-contained stub surface locking Store Milestone 4's public DX contract (§8)
//! and external-crate compile-fail authority fences (§11) in Phase M4.0.

mod home;
mod pin;

pub use home::*;
pub use pin::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResidencyBudget {
    pub head_working_set_bytes: u64,
    pub historical_working_set_bytes: u64,
    pub max_in_flight_fill_bytes: u64,
    pub streaming_window_bytes: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResidencyPressureSnapshot {
    pub head_resident_bytes: u64,
    pub head_borrowed_bytes: u64,
    pub historical_resident_bytes: u64,
    pub historical_borrowed_bytes: u64,
    pub raw_load_cache_bytes: u64,
    pub in_flight_fill_bytes: u64,
    pub evictions_completed: u64,
    pub busy_victim_skips: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReclaimBudget {
    max_bytes: u64,
}

impl ReclaimBudget {
    pub const fn bounded_bytes(max_bytes: u64) -> Self {
        Self { max_bytes }
    }

    pub const fn max_bytes(self) -> u64 {
        self.max_bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReclamationReport {
    pub reclaimed_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Principal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AccessScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MyApp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MySchema;

pub trait Program {
    type Schema;
}

impl Program for MyApp {
    type Schema = MySchema;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProductRevision(pub(crate) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommitId(pub(crate) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SchemaVersionIdentity(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RelationalStorageSubject(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalRootPointer(pub(crate) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtentAllocationGeneration {
    root_publication_generation: u64,
    extent_fence_seq: u64,
}

impl ExtentAllocationGeneration {
    pub const fn root_publication_generation(self) -> u64 {
        self.root_publication_generation
    }

    pub const fn extent_fence_seq(self) -> u64 {
        self.extent_fence_seq
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedBasisToken {
    pub(crate) commit_id: CommitId,
    pub(crate) root_semantic_commitment: [u8; 32],
    pub(crate) query_digest: [u8; 32],
}

impl RetainedBasisToken {
    pub const fn commit_id(&self) -> CommitId {
        self.commit_id
    }

    pub const fn root_semantic_commitment(&self) -> &[u8; 32] {
        &self.root_semantic_commitment
    }

    pub const fn query_digest(&self) -> &[u8; 32] {
        &self.query_digest
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryHint {
    MemoryPressure,
    CancelledOrDeadline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disposition {
    Complete,
    Final,
    Retry(RetryHint),
    Escalate,
}

pub trait Reportable {
    fn disposition(&self) -> Disposition;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticReport {
    pub(crate) disposition: Disposition,
}

impl DiagnosticReport {
    pub const fn disposition(&self) -> Disposition {
        self.disposition
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoricalReadDenial {
    HistoryUnavailable {
        commit_id: CommitId,
    },
    UnsupportedHistoricalMeaning {
        schema_version: SchemaVersionIdentity,
    },
    CorruptStorage {
        subject: RelationalStorageSubject,
    },
    MemoryExhausted {
        requested_bytes: u64,
        allowance_bytes: u64,
    },
    RequestStopped,
}

impl Reportable for HistoricalReadDenial {
    fn disposition(&self) -> Disposition {
        match self {
            Self::HistoryUnavailable { .. } | Self::UnsupportedHistoricalMeaning { .. } => {
                Disposition::Final
            }
            Self::CorruptStorage { .. } => Disposition::Escalate,
            Self::MemoryExhausted { .. } => Disposition::Retry(RetryHint::MemoryPressure),
            Self::RequestStopped => Disposition::Retry(RetryHint::CancelledOrDeadline),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    InvalidName,
    Historical(HistoricalReadDenial),
}

impl From<HistoricalReadDenial> for AppError {
    fn from(value: HistoricalReadDenial) -> Self {
        Self::Historical(value)
    }
}

/// Unauthenticated raw summary data — cannot directly authorize cold chunk skips.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChunkKindAdjacencySummary {
    pub kind_bits: u64,
}

/// Producer-verified summary sealed inside the chunk sealing owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedChunkSummary {
    summary: ChunkKindAdjacencySummary,
}

impl VerifiedChunkSummary {
    pub fn can_skip_kind(&self, mask: u64) -> bool {
        (self.summary.kind_bits & mask) == 0
    }
}

pub fn evaluate_cold_chunk_skip(summary: &VerifiedChunkSummary, required_kind_mask: u64) -> bool {
    summary.can_skip_kind(required_kind_mask)
}
