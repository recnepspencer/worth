//! every parent currentness drift stales retained authority.
use super::*;

#[test]
fn every_parent_currentness_drift_stales_retained_authority() {
    for drift in [
        ParentDrift::Revocation,
        ParentDrift::Expiry,
        ParentDrift::Workflow,
        ParentDrift::Resource,
        ParentDrift::DelegationGrantor,
        ParentDrift::Grantee,
        ParentDrift::PolicyPath,
        ParentDrift::Substitute,
    ] {
        let world = installed_delegated_capability_world();
        world.authorization_time.script([time(100), time(100)]);
        let request = live_scope();
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
        let access = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                admitted_capability_access(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                    &world,
                    &principal,
                    &request,
                    100,
                )
                .unwrap()
            })
            .expect("the fixture read owns its installed host call");
        world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                apply_parent_drift(
                    &host_custody.phase_for_application(&world.application),
                    &host_custody.borrowed_request_for_test(),
                    &world,
                    drift,
                )
            })
            .expect("the fixture read owns its installed host call");
        let operation = world
            .application
            .installed_schema()
            .installed_operation(
                super::super::super::fixture::CapabilityTouchOperation::reference(),
            )
            .unwrap();
        let Err(denial) = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                world.application.authorize_capability_operation(
                    &host_custody.phase_for_application(&world.application),
                    access,
                    &operation,
                    Default::default(),
                )
            })
            .expect("the fixture read owns its installed host call")
        else {
            panic!("retained authority must stale before a parent can be refreshed");
        };
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::StaleAuthorization,
            "drift {drift:?} did not invalidate the composite fact"
        );
    }
}
