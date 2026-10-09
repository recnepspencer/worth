//! Stale read set cleanup preserves the interleaved peer.

use super::*;

#[test]
fn stale_read_set_cleanup_preserves_the_interleaved_peer() {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    world.application.with_host_advancement(|active_phase| {
        let phase = &active_phase;

        let baseline = world.invariant.active_snapshot_count();
        let (victim, peer) = equivalent_programs(&world, "stale-interleaved");
        let victim = start(phase, &world.application, victim, idempotency(147, 148));
        let peer = start(phase, &world.application, peer, idempotency(149, 150));
        let both_attempts = world.invariant.active_snapshot_count();
        let winner = equivalent_programs(&world, "stale-winner").0;
        assert!(matches!(
            world
                .application
                .compare_and_commit_application_in_advancement(
                    phase,
                    winner,
                    idempotency(151, 152)
                ),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
        let victim = finish_application_commit(
            phase,
            &world.application,
            progress_application_commit(phase, &world.application, victim),
        );
        crate::domain_computation::primary_graph::tests::application_attempt::assert_product_basis_stale(
        victim,
        "the interleaved victim bound to the product before the winner",
    );
        assert_only_peer_remains(&world, baseline, both_attempts);
        finish_peer(
            phase,
            &world,
            peer,
            baseline,
            PeerExpectation::ProductBasisStale,
        );
    }).expect("fixture owner admits its advancement");
}
