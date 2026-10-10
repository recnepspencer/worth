use std::time::Duration;

use super::source_basis_is_admitted;
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::tests::{
    fixture::{installed_authorization_world, live_scope},
    live_delivery_support::commit_live_activity,
};
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

#[test]
fn owner_retained_program_basis_survives_a_new_head_without_admitting_foreign_or_direct_reads() {
    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.selected_product(&(phase).test_host_request())
        })
        .expect("the fixture read owns its installed host call");
    let source =
        WorthQueryProductBranchReadIdentity::from_observation(selected.product().observation());
    let retained = selected.retain_application_read();
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&(phase).test_host_request())
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .expect("the source owner authenticates")
        })
        .expect("the fixture read owns its installed host call");
    commit_live_activity(&world, &principal, &request);
    let current = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            WorthQueryProductBranchReadIdentity::from_observation(
                world
                    .selected_product(&(phase).test_host_request())
                    .product()
                    .observation(),
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_ne!(source, current);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(world
                .application
                .select_application_read_observation(&phase, &retained)
                .is_ok())
        })
        .expect("the fixture read owns its installed host call");
    assert!(source_basis_is_admitted(&source, &current, true));
    assert!(!source_basis_is_admitted(&source, &current, false));

    let foreign = installed_authorization_world(true);
    let foreign_current = foreign
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            WorthQueryProductBranchReadIdentity::from_observation(
                foreign
                    .selected_product(&(phase).test_host_request())
                    .product()
                    .observation(),
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(!source_basis_is_admitted(&source, &foreign_current, true));
    foreign
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(foreign
                .application
                .select_application_read_observation(&phase, &retained)
                .is_err())
        })
        .expect("the fixture read owns its installed host call");
}
