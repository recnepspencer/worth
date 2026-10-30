//! Generations whose ingest checkpointed resume frontiers. A frontier is
//! residue of the released session: it leaves with the release, before any
//! chunk, so the head still becomes terminal and retires.

#[allow(unused_imports)]
use crate::physical_blob_journeys::terminal_head_retirement::frontier::*;

/// The first batch takes the publication, the frontier and 62 chunks; the
/// second takes the last four chunks and then their root node.
#[test]
fn a_generation_with_a_resume_frontier_releases_to_a_terminal_head_and_retires() {
    release_and_retire(&LONG, &[(64, 64), (64, 5)]);
}

/// A batch too small for a chunk beside the frontier takes the frontier
/// alone, and the chunks follow in later batches.
#[test]
fn a_resume_frontier_leaves_alone_when_its_batch_has_no_room_for_a_chunk() {
    release_and_retire(&LONG, &[(1, 1), (1, 1), (64, 64), (64, 3)]);
}

/// A frontier of the released session keeps nothing: one batch with room
/// for the generation takes both frontiers, the chunk each names, the other
/// chunks and their root node.
#[test]
fn two_resume_frontiers_and_the_chunks_they_name_leave_in_one_batch() {
    release_and_retire(&TWO_FRONTIERS, &[(64, 8)]);
}

/// Batches too small for both frontiers split them: the publication, one
/// frontier alone, then the other frontier beside the first chunk.
#[test]
fn two_resume_frontiers_split_across_batches_both_leave_before_any_chunk() {
    release_and_retire(&TWO_FRONTIERS, &[(1, 1), (1, 1), (2, 2), (64, 4)]);
}
