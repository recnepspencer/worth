//! elevation status drift after admission is stale at operation progression.
use super::*;

#[test]
fn elevation_status_drift_after_admission_is_stale_at_operation_progression() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let (world, request, approved) = approval_transition::exact_approved_world(&host_custody);
        world.authorization_time.script([time(100), time(100)]);
        let principal = authenticated_principal(&host_custody, &world, &request);
        let access = admit(
            &host_custody,
            &world,
            &approved,
            &principal,
            &request,
            Some("elevation-2"),
        )
        .unwrap();
        mutation::set_status(
            &host_custody,
            &host_custody.borrowed_request_for_test(),
            &world,
            "elevation-2",
            CapabilityElevationStatus::Revoked,
        );
        let operation = world
            .application
            .installed_schema()
            .installed_operation(ElevatedCapabilityTouchOperation::reference())
            .unwrap();

        let Err(denial) = world.application.authorize_capability_operation(
            &host_custody.phase_for_application(&world.application),
            access,
            &operation,
            Default::default(),
        ) else {
            panic!("revoked elevation evidence must not progress to operation authority");
        };

        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
        );
    })
}
