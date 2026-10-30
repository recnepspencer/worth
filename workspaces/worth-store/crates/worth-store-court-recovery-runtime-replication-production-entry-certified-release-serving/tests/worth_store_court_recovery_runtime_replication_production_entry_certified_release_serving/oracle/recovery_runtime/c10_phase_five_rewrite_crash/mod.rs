use crate::c10_phase_five_rewrite_crash::kill_child;

/// Completing an inline rewrite reads its page from the source generation,
/// and from the successor too once the root publishes it.
#[test]
fn every_observation_limit_under_the_need_of_a_killed_inline_rewrite_is_that_limit() {
    for (seam, reads) in [("after-wal", 1), ("after-replace", 2)] {
        let (parent, root) = kill_child(seam);
        let blocks =
            crate::c10_phase_five_rewrite_crash::c10_crash_evidence::observation_sweep::completion_blocks_under_every_observation_limit(
                seam, &root,
            );
        assert!(
            blocks >= reads,
            "{seam}: completing the rewrite ran its reader out {blocks} times",
        );
        drop(parent);
    }
}
