//! Store-owned retained-basis historical cursor runtime (Store Milestone 4, Phase M4.4).
//!
//! Provides identical canonical-key resumable cursor semantics across both
//! Directory Homes ([`DurableHistoricalBasisStore`], backed by `PhysicalRootPins`
//! and `AnchoredRecords`) and Memory Homes ([`MemoryHistoricalBasisStore`],
//! backed by `VolatileRootPinTable`).
//!
//! Authority minting, token signing/advancement, and reclaim-fenced [`ReaderLoan`]
//! custody are strictly crate-private to `worth-store-application`.

mod authority;
mod aux_index;
mod basis_manifest;
mod basis_manifest_codec;
mod catalog;
mod chunk_codec;
mod chunk_format;
mod chunk_scan;
mod continuation_manifest;
mod cursor;
mod durable;
mod durable_io;
mod durable_manifest_io;
mod durable_mutations;
mod durable_query;
mod durable_reconcile;
mod durable_status;
mod memory;
mod memory_mutations;
mod memory_query;
mod memory_support;
mod persistent_map;
mod query_window;
mod relational_capture;
mod relational_capture_codec;
mod relational_capture_replay;
mod residency;
mod stream;
#[cfg(test)]
mod tests;
mod token;
mod wire_codec;

pub use chunk_format::{
    compute_chunk_semantic_commitment, compute_root_semantic_commitment,
    compute_row_history_digest, member_id, seal_basis_chunks, ChunkMemberId,
    HistoricalChunkGeometry, SealedBasisChunks, SealedChunkDescriptor, DEFAULT_CHUNK_ROWS,
    DEFAULT_CONTINUATION_PAGE_VERSIONS, DEFAULT_INLINE_HISTORY_LIMIT,
};
pub use chunk_scan::{consolidate_chunk, reconstruct_verified_chunk_rows};
pub use cursor::{
    BorrowedHistoricalRow, HistoricalCursor, HistoricalCursorPage, HistoricalCursorStream,
    ReaderLoan, ReaderLoanGuard,
};
pub use durable::DurableHistoricalBasisStore;
pub use durable_query::RetainedRevisionReceipt;
#[cfg(feature = "certification-test-authority")]
pub use durable_reconcile::DurableHistoricalCrashCut;
pub use memory::MemoryHistoricalBasisStore;
pub use query_window::{
    AdmittedQueryWindowBinding, AdmittedQueryWindowPage, AdmittedQueryWindowSource,
};
pub use relational_capture::{capture_relational_branch_revision, CapturedRelationalRevision};
pub use relational_capture_codec::{
    encode_canonical_entity_payload, encode_canonical_history_version,
    encode_canonical_relation_payload,
};
pub use residency::{HeadWorkingSetGuard, HistoricalPoolBorrow, HistoricalWorkingSetPool};
#[cfg(feature = "certification-test-authority")]
pub use stream::HistoricalCursorCertificationMutant;
pub use token::RetainedBasisToken;
