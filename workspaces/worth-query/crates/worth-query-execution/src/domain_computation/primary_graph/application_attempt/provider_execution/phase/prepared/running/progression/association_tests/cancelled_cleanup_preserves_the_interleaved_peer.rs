//! Cancelled cleanup preserves the interleaved peer.

use super::*;

#[test]
fn cancelled_cleanup_preserves_the_interleaved_peer() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let world =
            crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
                true,
            );
        world
            .application
            .with_host_advancement(|active_phase| {
                let phase = &active_phase;

                let baseline = world
                    .invariant
                    .active_snapshot_count(&active_phase.borrowed_request_for_test());
                let cancellation = WorthQueryCancellationSource::new();
                let request = WorthQueryRequestScope::new(
                    Instant::now() + Duration::from_secs(60),
                    cancellation.token(),
                );
                let principal = authenticated_principal(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                    &world,
                    &request,
                );
                let account = resolved_account(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                    &world,
                    "open",
                    &request,
                );
                let victim = retained_status_program(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                    &world,
                    &principal,
                    &account,
                    &request,
                    "cancelled-victim",
                    RetentionMutationBreadth::Narrow,
                );
                let peer = equivalent_programs(
                    &host_custody.phase_for_application(&world.application),
                    &world,
                    "cancelled-victim",
                )
                .0;
                let victim = start(phase, &world.application, victim, idempotency(141, 142));
                let peer = start(phase, &world.application, peer, idempotency(141, 142));
                let both_attempts = world
                    .invariant
                    .active_snapshot_count(&active_phase.borrowed_request_for_test());
                cancellation.cancel();
                let victim = finish_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    progress_application_commit(
                        &phase.borrowed_request_for_test(),
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
                assert_only_peer_remains(
                    &active_phase.borrowed_request_for_test(),
                    &world,
                    baseline,
                    both_attempts,
                );
                finish_peer(phase, &world, peer, baseline, PeerExpectation::Committed);
            })
            .expect("fixture owner admits its advancement");
    })
}
