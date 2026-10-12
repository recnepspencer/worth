//! width and cycle attacks are typed denials before access authority.
use super::*;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

#[test]
fn width_and_cycle_attacks_are_typed_denials_before_access_authority() {
    {
        let width = installed_delegated_capability_world();

        width.authorization_time.script([time(100)]);
        width
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                add_parent(
                    &host_custody,
                    &width,
                    "capability-child",
                    "capability-alternate",
                    "width-parent",
                )
            })
            .expect("the fixture read owns its installed host call");
        let request = live_scope();
        let principal = width
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                authenticated_principal(&host_custody, &width, &request)
            })
            .expect("the fixture read owns its installed host call");
        let Err(denial) = width
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                admitted_capability_access(&host_custody, &width, &principal, &request, 100)
            })
            .expect("the fixture read owns its installed host call")
        else {
            panic!("two current parents must deny");
        };
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::DelegationRejected
            )
        );

        let cycle = installed_delegated_capability_world();
        cycle.authorization_time.script([time(100)]);
        cycle
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                add_parent(
                    &host_custody,
                    &cycle,
                    "capability-grandparent",
                    "capability-child",
                    "cycle-parent",
                )
            })
            .expect("the fixture read owns its installed host call");
        let request = live_scope();
        let principal = cycle
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                authenticated_principal(&host_custody, &cycle, &request)
            })
            .expect("the fixture read owns its installed host call");
        let Err(denial) = cycle
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                admitted_capability_access(&host_custody, &cycle, &principal, &request, 100)
            })
            .expect("the fixture read owns its installed host call")
        else {
            panic!("a delegation cycle must deny before narrowing can be reused");
        };
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::DelegationCycle
            )
        );

        let depth = installed_delegated_capability_world();
        depth.authorization_time.script([time(100)]);
        depth
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                add_parent(
                    &host_custody,
                    &depth,
                    "capability-grandparent",
                    "capability-alternate",
                    "excess-depth-parent",
                )
            })
            .expect("the fixture read owns its installed host call");
        let request = live_scope();
        let principal = depth
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                authenticated_principal(&host_custody, &depth, &request)
            })
            .expect("the fixture read owns its installed host call");
        let Err(denial) = depth
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                admitted_capability_access(&host_custody, &depth, &principal, &request, 100)
            })
            .expect("the fixture read owns its installed host call")
        else {
            panic!("a chain beyond the installed depth must deny");
        };
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::DelegationDepthExceeded
            )
        );
    }
}
