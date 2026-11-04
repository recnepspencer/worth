//! Stale read set cleanup preserves the interleaved peer.

use crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progressed::association_tests::*;

#[test]
fn stale_read_set_cleanup_preserves_the_interleaved_peer() {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    world.application.with_host_advancement(|active_phase| {
let host_custody = active_phase.test_host_request_for_test();
        let phase = &active_phase;

        let baseline = world.invariant.active_snapshot_count_for_test(&active_phase.borrowed_request_for_test());
        let (victim, peer) = equivalent_programs(&host_custody.phase_for_application(&world.application), &world, "stale-interleaved");
        let victim = start(phase, &world.application, victim, idempotency(147, 148));
        let peer = start(phase, &world.application, peer, idempotency(149, 150));
        let both_attempts = world.invariant.active_snapshot_count_for_test(&active_phase.borrowed_request_for_test());
        let winner = equivalent_programs(&host_custody.phase_for_application(&world.application), &world, "stale-winner").0;
        assert!(matches!(
            world
                .application
                .compare_and_commit_application_in_advancement(
                    phase,
                    winner,
                    idempotency(151, 152)
                , worth_execution::ExecutionAllocationPolicy::SystemAllocation),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
        let victim = finish_application_commit(&phase.borrowed_request_for_test(),
            phase,
            &world.application,
            progress_application_commit(&phase.borrowed_request_for_test(), phase, &world.application, victim, None, worth_execution::ExecutionAllocationPolicy::SystemAllocation),
        );
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::decision_currentness::assert_product_basis_stale(
        victim,
        "the interleaved victim bound to the product before the winner",
    );
        assert_only_peer_remains(&active_phase.borrowed_request_for_test(), &world, baseline, both_attempts);
        finish_peer(
            phase,
            &world,
            peer,
            baseline,
            PeerExpectation::ProductBasisStale,
        );
    }).expect("fixture owner admits its advancement");
}
