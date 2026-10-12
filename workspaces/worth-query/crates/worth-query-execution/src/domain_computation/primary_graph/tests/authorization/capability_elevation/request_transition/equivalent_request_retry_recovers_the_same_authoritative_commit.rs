//! equivalent request retry recovers the same authoritative commit.
use super::*;

#[test]
fn equivalent_request_retry_recovers_the_same_authoritative_commit() {
    let world = request_world();
    let request = live_scope();
    {
        let principal = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                authenticated_principal(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                    &world,
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let first = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                request_reads(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
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
                let host_custody = phase.test_host_request();
                request_reads(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
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
            WorthQueryElevationRequestOutcome::AlreadyRequested(recovered) => recovered,
            unexpected => {
                panic!("equivalent request retry must recover the original commit: {unexpected:?}")
            }
        };
        assert!(recovered
            .commit_receipt()
            .is_same_authoritative_commit(original.commit_receipt()));
    }
}
