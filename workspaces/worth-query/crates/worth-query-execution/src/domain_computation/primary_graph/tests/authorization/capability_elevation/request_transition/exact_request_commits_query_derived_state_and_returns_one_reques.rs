//! exact request commits query derived state and returns one requested receipt.
use super::*;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

#[test]
fn exact_request_commits_query_derived_state_and_returns_one_requested_receipt() {
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
        let program = world
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

        let outcome = world
            .application
            .compare_and_commit_elevation_request(program, idempotency(71, 71));
        let WorthQueryElevationRequestOutcome::Requested(receipt) = outcome else {
            panic!("the exact request transition must commit: {outcome:?}");
        };

        assert_eq!(receipt.commit_receipt().changed_record_count(), 8);
        assert_eq!(receipt.commit_receipt().emitted_effect_count(), 0);
        assert_eq!(receipt.elevation_key(), "elevation-request-2");
        assert_eq!(receipt.review_key(), "review-request-2");
        assert_eq!(receipt.requester(), principal.principal_entity_id());
        assert_eq!(receipt.elevation_identity(), &string("elevation-2"));
        assert_eq!(receipt.reason(), &string("protect-customer"));
        assert_eq!(receipt.requested_status(), &string("requested"));
        assert_eq!(receipt.issued_at(), &AspectValue::UInt64(100));
        assert_eq!(receipt.expires_at(), &AspectValue::UInt64(105));
        assert_eq!(receipt.review_identity(), &string("review-2"));
        assert_eq!(receipt.review_status(), &string("required"));

        world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                resolve_created_identities(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                    &host_custody.borrowed_request_for_test(),
                    &world,
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        world.authorization_time.script([time(100)]);
        let denial = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                super::super::admit_raw(
                    &host_custody.phase_for_application(&world.application),
                    &world,
                    &principal,
                    &request,
                    Some("elevation-2"),
                )
                .err()
                .expect("a requested selector cannot bypass the lifecycle transition")
            })
            .expect("the fixture read owns its installed host call");
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ElevationTransitionRequired
            )
        );
    }
}
