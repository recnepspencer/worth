use crate::c10_phase_six_span_crash::kill_span;

/// Completing a span rewrite reads the whole source span, and the successor
/// span too once the root publishes it.
#[test]
fn every_observation_limit_under_the_need_of_a_killed_span_rewrite_is_that_limit() {
    for (seam, reads) in [("after-wal", 1), ("after-replace", 2)] {
        let (parent, root) = kill_span(4, 4, seam);
        let blocks =
            crate::c10_phase_six_span_crash::c10_crash_evidence::observation_sweep::completion_blocks_under_every_observation_limit(
                seam, &root,
            );
        assert!(
            blocks >= reads,
            "{seam}: completing the rewrite ran its reader out {blocks} times",
        );
        drop(parent);
    }
}
