//! Chunked placement courts (Store M4.3a/M4.3c: `S2`, `M3`).
mod member_store;

mod chunk_partition;
mod consolidation_invariance;
mod paged_directory_reopen;
mod placement_hardening;
mod region_placement;
mod sealing_refusals;
mod single_slot_fanout;
mod summary_pruning;
mod value_streaming;
