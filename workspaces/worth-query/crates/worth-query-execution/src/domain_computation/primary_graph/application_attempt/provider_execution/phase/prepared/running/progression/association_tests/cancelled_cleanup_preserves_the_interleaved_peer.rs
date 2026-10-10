//! Cancelled cleanup preserves the interleaved peer.

use super::*;

#[test]
fn cancelled_cleanup_preserves_the_interleaved_peer() {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;

            let baseline = world.invariant.active_snapshot_count();
            let cancellation = WorthQueryCancellationSource::new();
            let request = WorthQueryRequestScope::new(
                Instant::now() + Duration::from_secs(60),
                cancellation.token(),
            );
            let principal = authenticated_principal(&world, &request);
            let account = resolved_account(&world, "open", &request);
            let victim = retained_status_program(
                &world,
                &principal,
                &account,
                &request,
                "cancelled-victim",
                RetentionMutationBreadth::Narrow,
            );
            let peer = equivalent_programs(&world, "cancelled-victim").0;
            let victim = start(phase, &world.application, victim, idempotency(141, 142));
            let peer = start(phase, &world.application, peer, idempotency(141, 142));
            let both_attempts = world.invariant.active_snapshot_count();
            cancellation.cancel();
            let victim = finish_application_commit(
                phase,
                &world.application,
                progress_application_commit(
                    phase,
                    &world.application,
                    victim,
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                ),
            );
            assert!(
                matches!(victim, WorthQueryApplicationCommitOutcome::Cancelled),
                "cancelled victim must remain cancelled, got {victim:?}"
            );
            assert_only_peer_remains(&world, baseline, both_attempts);
            finish_peer(phase, &world, peer, baseline, PeerExpectation::Committed);
        })
        .expect("fixture owner admits its advancement");
}
