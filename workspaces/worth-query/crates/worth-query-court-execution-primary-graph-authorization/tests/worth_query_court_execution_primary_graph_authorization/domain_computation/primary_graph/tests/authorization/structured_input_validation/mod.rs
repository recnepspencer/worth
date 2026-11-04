use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityTouchOperation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountCapability;
use crate::domain_computation::primary_graph::tests::authorization::capability_progression::capability_input;
use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;

#[test]
fn structured_input_validation_is_enforced_by_capability_admission() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let capability = world
        .application
        .installed_schema()
        .capability(
            TouchAccountCapability::reference(),
            CapabilityTouchOperation::reference(),
        )
        .unwrap();
    let input = capability_input(u64::MAX);

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .admit_capability_access(&phase, &principal, &capability, input, &request)
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("an invalid structured input must not mint capability authority")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::InvalidOperationInput
        )
    );
}
