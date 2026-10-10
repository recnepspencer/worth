//! zero or overlong duration and invalid storage key fail before commit.
use super::*;

#[test]
fn zero_or_overlong_duration_and_invalid_storage_key_fail_before_commit() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        for duration in [Duration::ZERO, Duration::from_secs(1_201)] {
            let world = request_world();
            let request = live_scope();
            let principal = authenticated_principal(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                &request,
            );
            let access = request_access(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                &principal,
                &request,
                RequestElevationInput {
                    duration,
                    ..honest_input()
                },
            )
            .unwrap();
            let denial = world
                .application
                .authorize_elevation_request(
                    &host_custody.phase_for_application(&world.application),
                    access,
                    &request_operation(&world),
                    Default::default(),
                )
                .err()
                .expect("invalid duration must not mint request authority");
            assert_eq!(
                denial.kind(),
                WorthQueryOperationAuthorizationDenialKind::ElevationDurationExceeded
            );
        }

        let world = request_world();
        let request = live_scope();
        let principal = authenticated_principal(
            &(host_custody.phase_for_application(&world.application)).test_host_request(),
            &world,
            &request,
        );
        let access = request_access(
            &(host_custody.phase_for_application(&world.application)).test_host_request(),
            &world,
            &principal,
            &request,
            RequestElevationInput {
                elevation_key: " bad-key".to_owned(),
                ..honest_input()
            },
        )
        .unwrap();
        let denial = world
            .application
            .authorize_elevation_request(
                &host_custody.phase_for_application(&world.application),
                access,
                &request_operation(&world),
                Default::default(),
            )
            .err()
            .expect("invalid provider key must fail before lifecycle graph progression");
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::ElevationRequestRejected
        );
    })
}
