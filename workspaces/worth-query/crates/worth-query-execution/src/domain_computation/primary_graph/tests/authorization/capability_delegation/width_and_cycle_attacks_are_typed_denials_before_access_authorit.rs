//! width and cycle attacks are typed denials before access authority.
use super::*;

#[test]
fn width_and_cycle_attacks_are_typed_denials_before_access_authority() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let width = installed_delegated_capability_world();
        width.authorization_time.script([time(100)]);
        add_parent(
            &host_custody,
            &width,
            "capability-child",
            "capability-alternate",
            "width-parent",
        );
        let request = live_scope();
        let principal = authenticated_principal(&host_custody, &width, &request);
        let Err(denial) =
            admitted_capability_access(&host_custody, &width, &principal, &request, 100)
        else {
            panic!("two current parents must deny");
        };
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::DelegationRejected
        );

        let cycle = installed_delegated_capability_world();
        cycle.authorization_time.script([time(100)]);
        add_parent(
            &host_custody,
            &cycle,
            "capability-grandparent",
            "capability-child",
            "cycle-parent",
        );
        let request = live_scope();
        let principal = authenticated_principal(&host_custody, &cycle, &request);
        let Err(denial) =
            admitted_capability_access(&host_custody, &cycle, &principal, &request, 100)
        else {
            panic!("a delegation cycle must deny before narrowing can be reused");
        };
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::DelegationCycle
        );

        let depth = installed_delegated_capability_world();
        depth.authorization_time.script([time(100)]);
        add_parent(
            &host_custody,
            &depth,
            "capability-grandparent",
            "capability-alternate",
            "excess-depth-parent",
        );
        let request = live_scope();
        let principal = authenticated_principal(&host_custody, &depth, &request);
        let Err(denial) =
            admitted_capability_access(&host_custody, &depth, &principal, &request, 100)
        else {
            panic!("a chain beyond the installed depth must deny");
        };
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::DelegationDepthExceeded
        );
    })
}
