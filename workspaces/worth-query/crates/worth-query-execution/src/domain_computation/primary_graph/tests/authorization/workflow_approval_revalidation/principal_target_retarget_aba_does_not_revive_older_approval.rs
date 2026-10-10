//! principal target retarget aba does not revive older approval.
use super::*;

#[test]
fn principal_target_retarget_aba_does_not_revive_older_approval() {
    let world = installed_capability_live_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let admission = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_operation(
                &(phase).test_host_request(),
                &world,
                &principal,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let basis = durable_approval_basis(&admission);
    let binding = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout()
        .principal_binding(world.binding.binding())
        .unwrap();
    let other = world
        .application
        .with_host_advancement(|phase| {
            world
        .selected_product(&(phase).test_host_request(), )
        .resolve_entity(&phase,
            super::super::super::fixture::PrincipalIdentityField::reference(),
            2_u64,
            &request,
            crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id()
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            retarget_relation_without_replacing_id(
                &host_custody.borrowed_request_for_test(),
                &world,
                principal.target_relation_id(),
                binding.relation_kind,
                principal.mapping_entity_id(),
                other,
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            retarget_relation_without_replacing_id(
                &host_custody.borrowed_request_for_test(),
                &world,
                principal.target_relation_id(),
                binding.relation_kind,
                principal.mapping_entity_id(),
                principal.principal_entity_id(),
            )
        })
        .expect("the fixture read owns its installed host call");
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
        WorthQueryOperationAuthorizationDenialKind::StalePrincipal
    );
}
