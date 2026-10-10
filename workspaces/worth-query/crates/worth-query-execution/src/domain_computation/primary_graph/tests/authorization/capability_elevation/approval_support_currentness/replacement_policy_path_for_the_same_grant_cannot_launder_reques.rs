//! replacement policy path for the same grant cannot launder request support.
use super::*;

#[test]
fn replacement_policy_path_for_the_same_grant_cannot_launder_request_support() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let borrowed_execution = host_custody.borrowed_request_for_test();
        let execution = &borrowed_execution;

        let (world, request, requested) = super::super::approval_transition::requested_world(
            &host_custody,
            CapabilityElevationScenario::Active,
        );
        let requester = super::super::approval_transition::authenticated(
            &host_custody,
            &world,
            "alice",
            &request,
        );
        super::super::mutation::replace_support_grantor_with_custodian(
            &host_custody,
            execution,
            &world,
            requester.principal_entity_id(),
        );
        let approver = super::super::approval_transition::authenticated(
            &host_custody,
            &world,
            "bob",
            &request,
        );
        let access = super::super::approval_transition::approval_access(
            &host_custody,
            &world,
            &approver,
            &request,
        )
        .expect("the independent approval command grant remains current");

        let Err(denial) = world.application.authorize_elevation_approval(
            &host_custody.phase_for_application(&world.application),
            requested,
            access,
            &super::super::approval_transition::approval_operation(&world),
            Default::default(),
        ) else {
            panic!("a new allowing path must not replace the request receipt's original path");
        };

        assert_eq!(
            denial.denial().kind(),
            WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
        );
        assert_requested_without_approval(
            &host_custody.phase_for_application(&world.application),
            &host_custody.borrowed_request_for_test(),
            &world,
            &denial.into_requested(),
            approver.principal_entity_id(),
        );
    })
}
