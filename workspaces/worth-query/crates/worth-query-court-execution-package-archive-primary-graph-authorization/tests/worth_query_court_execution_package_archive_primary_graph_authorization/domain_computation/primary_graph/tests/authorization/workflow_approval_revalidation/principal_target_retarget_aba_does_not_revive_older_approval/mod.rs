//! principal target retarget aba does not revive older approval.
use crate::domain_computation::primary_graph::tests::authorization::workflow_approval_revalidation::*;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

#[test]
fn principal_target_retarget_aba_does_not_revive_older_approval() {
    let world = installed_capability_live_world();
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
    let binding = (*world
        .application
        .runtime_for_test())
        .primary_graph()
        .unwrap()
        .layout()
        .principal_binding(world.binding.binding())
        .unwrap();
    let other = world
        .application
        .with_host_advancement(|phase| {
            world
        .selected_product(&(phase).test_host_request_for_test(), )
        .resolve_entity(&phase,
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::PrincipalIdentityField::reference(),
            2_u64,
            &request,
            worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id_for_test()
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            retarget_relation_without_replacing_id(
                &host_custody.borrowed_request_for_test(),
                &world,
                principal.target_relation_id_for_test(),
                binding.relation_kind,
                principal.mapping_entity_id_for_test(),
                other,
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            retarget_relation_without_replacing_id(
                &host_custody.borrowed_request_for_test(),
                &world,
                principal.target_relation_id_for_test(),
                binding.relation_kind,
                principal.mapping_entity_id_for_test(),
                principal.principal_entity_id_for_test(),
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
        WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::StalePrincipal)
    );
}
