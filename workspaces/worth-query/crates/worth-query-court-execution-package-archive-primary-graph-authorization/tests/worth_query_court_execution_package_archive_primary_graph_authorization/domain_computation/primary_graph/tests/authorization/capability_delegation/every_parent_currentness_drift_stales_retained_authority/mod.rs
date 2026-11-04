//! every parent currentness drift stales retained authority.
use crate::domain_computation::primary_graph::tests::authorization::capability_delegation::*;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

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
                let host_custody = phase.test_host_request_for_test();
                authenticated_principal(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                    &world,
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let access = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                admitted_capability_access(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
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
                let host_custody = phase.test_host_request_for_test();
                apply_parent_drift(
                    &host_custody.phase_for_application_for_test(&world.application),
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
                worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityTouchOperation::reference(),
            )
            .unwrap();
        let Err(denial) = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                world.application.authorize_capability_operation(
                    &host_custody.phase_for_application_for_test(&world.application),
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
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::StaleAuthorization
            ),
            "drift {drift:?} did not invalidate the composite fact"
        );
    }
}
