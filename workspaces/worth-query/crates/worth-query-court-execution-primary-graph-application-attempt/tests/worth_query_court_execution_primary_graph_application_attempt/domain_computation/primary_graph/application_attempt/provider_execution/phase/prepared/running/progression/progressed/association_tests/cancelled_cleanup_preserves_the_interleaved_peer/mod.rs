//! Cancelled cleanup preserves the interleaved peer.

use crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progressed::association_tests::*;

#[test]
fn cancelled_cleanup_preserves_the_interleaved_peer() {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    world
        .application
        .with_host_advancement(|active_phase| {
            let host_custody = active_phase.test_host_request_for_test();
            let phase = &active_phase;

            let baseline = world
                .invariant
                .active_snapshot_count_for_test(&active_phase.borrowed_request_for_test());
            let cancellation = WorthQueryCancellationSource::new();
            let request = WorthQueryRequestScope::new(
                Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(60)),
                cancellation.token(), worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
            let principal = authenticated_principal(
                &(host_custody.phase_for_application_for_test(&world.application))
                    .test_host_request_for_test(),
                &world,
                &request,
            );
            let account = resolved_account(
                &(host_custody.phase_for_application_for_test(&world.application))
                    .test_host_request_for_test(),
                &world,
                "open",
                &request,
            );
            let victim = retained_status_program(
                &(host_custody.phase_for_application_for_test(&world.application))
                    .test_host_request_for_test(),
                &world,
                &principal,
                &account,
                &request,
                "cancelled-victim",
                RetentionMutationBreadth::Narrow,
            );
            let peer = equivalent_programs(
                &host_custody.phase_for_application_for_test(&world.application),
                &world,
                "cancelled-victim",
            )
            .0;
            let victim = start(phase, &world.application, victim, idempotency(141, 142));
            let peer = start(phase, &world.application, peer, idempotency(141, 142));
            let both_attempts = world
                .invariant
                .active_snapshot_count_for_test(&active_phase.borrowed_request_for_test());
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
                    None,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
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
}
