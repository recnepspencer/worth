use super::*;

#[test]
fn resource_and_grantor_grantee_link_drift_deny_at_the_transition() {
    {
        let resource = installed_delegated_capability_world();

        resource.authorization_time.script([time(100)]);
        resource
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                replace_parent_resource(&host_custody, &resource)
            })
            .expect("the fixture read owns its installed host call");
        let request = live_scope();
        let principal = resource
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                authenticated_principal(&host_custody, &resource, &request)
            })
            .expect("the fixture read owns its installed host call");
        let Err(denial) = resource
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                admitted_capability_access(&host_custody, &resource, &principal, &request, 100)
            })
            .expect("the fixture read owns its installed host call")
        else {
            panic!("a parent bound to another resource must deny");
        };
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::DelegationRejected
        );

        let actors = installed_delegated_capability_world();
        actors.authorization_time.script([time(100)]);
        actors
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                replace_child_grantor_with_grantee(
                    &host_custody,
                    &host_custody.borrowed_request_for_test(),
                    &actors,
                )
            })
            .expect("the fixture read owns its installed host call");
        let request = live_scope();
        let principal = actors
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                authenticated_principal(&host_custody, &actors, &request)
            })
            .expect("the fixture read owns its installed host call");
        let Err(denial) = actors
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                admitted_capability_access(&host_custody, &actors, &principal, &request, 100)
            })
            .expect("the fixture read owns its installed host call")
        else {
            panic!("a child grantor who is not the parent grantee must deny");
        };
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::DelegationRejected
        );
    }
}
