//! approval command cannot select a different elevation subject.
use super::*;

#[test]
fn approval_command_cannot_select_a_different_elevation_subject() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let (world, request, requested) =
            requested_world(&host_custody, CapabilityElevationScenario::Active);
        let approver = authenticated(&host_custody, &world, "bob", &request);
        let access = approval_access_for_elevation(
            &host_custody,
            &world,
            &approver,
            &request,
            "elevation-1",
        )
        .expect("the approver independently holds command authority for elevation-1");

        let denial = world
            .application
            .authorize_elevation_approval(
                &host_custody.phase_for_application(&world.application),
                requested,
                access,
                &approval_operation(&world),
                Default::default(),
            )
            .err()
            .expect("command authority for another subject must not consume this request");
        assert_eq!(
            denial.denial().kind(),
            WorthQueryOperationAuthorizationDenialKind::ElevationApprovalRejected
        );
    })
}
