//! Pure data vocabulary types for retained-basis historical cursors
//! (Store Milestone 4, Phase M4.4).
//!
//! These types carry zero execution or minting authority. Token minting,
//! reclaim-fenced [`ReaderLoan`]s, and [`HistoricalCursor`]s are owned and
//! sealed exclusively inside `worth-store-application`.

use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::{KindId, PartitionId};

use super::key::{
    BranchOriginIdentity, CanonicalCursorKey, CanonicalKeyDomain, SchemaVersionIdentity,
};

/// Current wire format version for `RetainedBasisToken`.
pub const RETAINED_BASIS_TOKEN_VERSION: u16 = 1;
/// Canonical opaque wire prefix for `RetainedBasisToken`.
pub const RETAINED_BASIS_TOKEN_PREFIX: &str = "whc1_";

/// Independent durable retention class keeping a historical basis reachable
/// across process restarts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HistoricalBasisRetentionKind {
    /// Selected root of a live branch.
    LiveBranch,
    /// Selected root of an archived branch (read-only historical retention).
    ArchivedBranch,
    /// Explicitly pinned via Store's `PinTable` / `BranchLease`.
    PinLease,
    /// Retained within the Home's configured retention horizon.
    RetentionHorizon,
}

/// One canonical row yielded in canonical logical key order from a retained basis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricalBasisRow {
    pub partition_id: PartitionId,
    pub domain: CanonicalKeyDomain,
    pub kind_id: KindId,
    pub logical_key: Vec<u8>,
    pub payload: Vec<u8>,
    pub history_versions: Vec<Vec<u8>>,
}

impl HistoricalBasisRow {
    /// Canonical logical cursor position strictly after this row.
    pub fn canonical_cursor_key(&self) -> Option<CanonicalCursorKey> {
        CanonicalCursorKey::after(self.partition_id, self.domain, self.logical_key.clone())
    }
}

/// Residency observation separating the `Historical` pool from the `Head` pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HistoricalResidencyPoolSnapshot {
    pub head_resident_bytes: u64,
    pub head_allowance_bytes: u64,
    pub historical_resident_bytes: u64,
    pub historical_borrowed_bytes: u64,
    pub historical_allowance_bytes: u64,
    pub in_flight_fill_bytes: u64,
    pub historical_evictions: u64,
    pub chunks_faulted: u64,
    pub continuation_pages_faulted: u64,
}

/// Owner-side assembly fields carried by a `RetainedBasisToken`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedBasisTokenFields {
    pub home_identity: HomeIdentity,
    pub home_incarnation: HomeIncarnation,
    pub branch_origin: BranchOriginIdentity,
    pub commit_id: CommitId,
    pub root_semantic_commitment: [u8; 32],
    pub schema_version: SchemaVersionIdentity,
    pub query_digest: [u8; 32],
    pub canonical_key_position: CanonicalCursorKey,
}
