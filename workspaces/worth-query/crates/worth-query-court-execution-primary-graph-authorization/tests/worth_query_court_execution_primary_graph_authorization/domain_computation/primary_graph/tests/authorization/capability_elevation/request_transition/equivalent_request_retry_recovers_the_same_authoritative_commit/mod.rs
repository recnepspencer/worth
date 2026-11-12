//! equivalent request retry recovers the same authoritative commit.
use crate::domain_computation::primary_graph::tests::authorization::capability_elevation::request_transition::*;

#[test]
fn equivalent_request_retry_recovers_original_data_without_elevation_authority() {
    let world = request_world();
    let request = live_scope();
    {
        let principal = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                authenticated_principal(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                    &world,
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let first = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                request_reads(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                    &world,
                    &principal,
                    &request,
                    honest_input(),
                )
                .materialize_elevation_request_program()
                .unwrap()
            })
            .expect("the fixture read owns its installed host call");
        let retry = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                request_reads(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                    &world,
                    &principal,
                    &request,
                    honest_input(),
                )
                .materialize_elevation_request_program()
                .unwrap()
            })
            .expect("the fixture read owns its installed host call");

        let first_outcome = world
            .application
            .compare_and_commit_elevation_request(first, idempotency(73, 73));
        let original = match first_outcome {
            WorthQueryElevationRequestOutcome::Requested(original) => original,
            unexpected => panic!("first request must commit: {unexpected:?}"),
        };
        let retry_outcome = world
            .application
            .compare_and_commit_elevation_request(retry, idempotency(73, 73));
        let recovered = match retry_outcome {
            WorthQueryElevationRequestOutcome::RecoveredCommitted(recovered) => recovered,
            unexpected => {
                panic!("equivalent request retry must recover the original commit: {unexpected:?}")
            }
        };
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::recorded_receipt::assert_original(
            &recovered,
            original.commit_receipt(),
        );
    }
}
