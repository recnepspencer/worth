//! Chunked placement groups (Store M4.3a substrate for `CHK24`/`REG8`).
//!
//! A placement group keeps its `PartitionId` and slot numbers; its slots are
//! cut into fixed, non-overlapping [`slot_range::SlotRange`] chunks per slot
//! domain. Each chunk is sealed once by [`sealing::seal_chunk_image`], the
//! single producer owner, into two separately authenticated facts:
//!
//! - [`semantic_entry::ChunkSemanticEntry`]: the consolidation-invariant
//!   commitment over the chunk's canonical meaning (head and history records,
//!   folded per-slot continuations, producer-derived summaries). Ordered
//!   entries fold into `root_semantic_merkle`.
//! - [`locator_binding::ChunkLocatorBinding`]: the physical members holding
//!   that meaning. History consolidation replaces its `history_locator` only
//!   after re-deriving a bit-identical semantic commitment.
//!
//! Slot histories and adjacency lists above the layout's inline budget spill
//! into content-addressed continuation pages indexed by a
//! [`paged_directory`], faulted and evicted page by page through the home's
//! `WorkingSetAccountant`. Large field values stream through
//! [`value_reader::ValueReader`] windows charged to the in-flight fill ledger.
//!
//! Every `CHK24` region leaf records a [`region_placement::RegionChunkPlacement`]
//! sealed by [`region_projection::seal_region_chunks`]; the manifest pins its
//! member roster and admission validates it. Cold readers open it through
//! [`region_reader::RecordedRegionChunks`].

pub mod canonical_read;
pub mod chunk_directory;
pub mod chunk_images;
pub mod chunk_summary;
pub mod cold_member_port;
pub mod consolidation;
pub mod continuation;
pub mod layout;
pub mod locator_binding;
pub mod member_frame;
pub mod member_io;
pub mod page_cache;
pub mod paged_directory;
pub mod region_placement;
pub mod region_projection;
pub mod region_reader;
pub mod sealing;
pub mod semantic_entry;
pub mod slot_projection;
pub mod slot_range;
pub mod summary_scan;
pub mod value_reader;

#[cfg(test)]
mod tests;
