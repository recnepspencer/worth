//! support expiry denies approval while the request lifecycle is current.
use super::*;

#[test]
fn support_expiry_denies_approval_while_the_request_lifecycle_is_current() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let (world, request, requested) = super::super::approval_transition::requested_world(
            &host_custody,
            CapabilityElevationScenario::ExpiringSupport,
        );
        world
            .authorization_time
            .script(std::iter::repeat_n(time(103), 8));
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
        .expect("the approval command and request lifecycle remain current at time 103");

        let Err(denial) = world.application.authorize_elevation_approval(
            &host_custody.phase_for_application(&world.application),
            requested,
            access,
            &super::super::approval_transition::approval_operation(&world),
            Default::default(),
        ) else {
            panic!("the exact support expired at time 102 and must be reobserved");
        };

        assert_eq!(
            denial.denial().kind(),
            WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing
        );
        assert!(denial.denial().identity().is_some());
        assert_eq!(
            denial.denial().causes(),
            [WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing]
        );
        let requested = denial.into_requested();
        assert_eq!(
            requested.expires_at(),
            &worth_foundational::facade::AspectValue::UInt64(105)
        );
        assert_requested_without_approval(
            &host_custody.phase_for_application(&world.application),
            &host_custody.borrowed_request_for_test(),
            &world,
            &requested,
            approver.principal_entity_id(),
        );
    })
}
