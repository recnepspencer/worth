//! Abandoned running attempt preserves the interleaved peer.

use super::*;

#[test]
fn abandoned_running_attempt_preserves_the_interleaved_peer() {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;

            let baseline = world.invariant.active_snapshot_count();
            let (victim, peer) = equivalent_programs(&world, "abandoned-victim");
            let victim = start(phase, &world.application, victim, idempotency(143, 144));
            let peer = start(phase, &world.application, peer, idempotency(143, 144));
            let both_attempts = world.invariant.active_snapshot_count();
            drop(victim);
            assert_only_peer_remains(&world, baseline, both_attempts);
            finish_peer(phase, &world, peer, baseline, PeerExpectation::Committed);
        })
        .expect("fixture owner admits its advancement");
}
