//! Completing a killed extent rewrite reads whole extent generations. Under
//! fewer observation bytes than those reads need, recovery is told that
//! limit, never that the extent is damaged.

use super::*;
use c10_crash_evidence::observation_sweep::completion_blocks_under_every_observation_limit;

#[test]
fn every_observation_limit_under_the_need_of_a_killed_extent_rewrite_is_that_limit() {
    // After the WAL the rewrite is applied from its source generation; after
    // the root replacement the selected successor is proved against it.
    for seam in ["after-wal", "after-replace"] {
        let (parent, root) = kill_child(seam);
        let blocks = completion_blocks_under_every_observation_limit(seam, &root);
        // Each of the payload's three chunks is one read, refused once.
        assert!(
            blocks >= 3,
            "{seam}: completing the rewrite ran its reader out {blocks} times",
        );
        drop(parent);
    }
}
