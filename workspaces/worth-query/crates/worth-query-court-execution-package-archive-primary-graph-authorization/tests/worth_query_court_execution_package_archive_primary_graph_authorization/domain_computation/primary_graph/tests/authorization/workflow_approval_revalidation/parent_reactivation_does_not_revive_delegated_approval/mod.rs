//! parent reactivation does not revive delegated approval.
use crate::domain_computation::primary_graph::tests::authorization::workflow_approval_revalidation::*;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

#[test]
fn parent_reactivation_does_not_revive_delegated_approval() {
    let world = installed_delegated_capability_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let admission = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_operation(
                &(phase).test_host_request_for_test(),
                &world,
                &principal,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let basis = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            durable_approval_basis(&host.borrowed_request_for_test(), &admission)
        })
        .expect("the fixture envelope owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| revoke_parent(&phase, &world))
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            update_grant_field(
                &phase,
                &world,
                "capability-parent",
                field(
                    &world,
                    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityStatusField::reference(),
                ),
                worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityStatusBinding::encode(
                    &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityStatus::Active,
                )
                .unwrap(),
            )
        })
        .expect("the fixture read owns its installed host call");
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .readmit_workflow_approval_authority(
                    &host_custody.borrowed_request_for_test(),
                    &basis,
                    &admission,
                )
                .unwrap_err()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::StaleAuthorization
        )
    );
}
