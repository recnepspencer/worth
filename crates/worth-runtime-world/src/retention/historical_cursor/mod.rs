//! Pure contract vocabulary for retained-basis historical cursors
//! (Store Milestone 4, Phase M4.4).
//!
//! Runtime cursor admission, Home-MAC token minting, and reclaim-fenced
//! `ReaderLoan` custody are strictly sealed inside `worth-store-application`
//! so external safe Rust has zero path to forge tokens, mint an authority, or
//! fabricate loan custody.

mod denial;
mod key;
#[cfg(test)]
mod tests;
mod types;

pub use denial::{HistoricalReadDenial, RetainedBasisTokenDecodeError};
pub use key::{
    AdmittedHistoricalProjection, BranchOriginIdentity, CanonicalCursorKey, CanonicalKeyDomain,
    HistoricalQueryContract, SchemaVersionIdentity, MAX_BRANCH_ID_BYTES,
    MAX_CANONICAL_CURSOR_KEY_BYTES, MAX_SCHEMA_ID_BYTES, ORDERING_CANONICAL_ASC,
    PROJECTION_CANONICAL_FULL, PROJECTION_CANONICAL_HEAD_ONLY, PROJECTION_CANONICAL_KEYS_ONLY,
    PROJECTION_QUERY_ROWS,
};
pub use types::{
    HistoricalBasisRetentionKind, HistoricalBasisRow, HistoricalResidencyPoolSnapshot,
    RetainedBasisTokenFields, RETAINED_BASIS_TOKEN_PREFIX, RETAINED_BASIS_TOKEN_VERSION,
};
