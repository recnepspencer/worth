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
        .with_host_advancement(|host_phase| world.selected_product(&host_phase.test_host_request()))
        .expect("installed host call opens");
    let source =
        WorthQueryProductBranchReadIdentity::from_observation(selected.product().observation());
    let retained = selected.retain_application_read();
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|host_phase| {
            world
                .selected_product(&host_phase.test_host_request())
                .resolve_authenticated_principal(
                    &host_phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .expect("the source owner authenticates")
        })
        .expect("installed host call opens");
    commit_live_activity(&world, &principal, &request);
    let selected_current = world
        .application
        .with_host_advancement(|host_phase| world.selected_product(&host_phase.test_host_request()))
        .expect("installed host call opens");
    let current = selected_current.product().observation();
    assert!(!source.matches_observation(current));
    assert!(world
        .application
        .with_host_advancement(|host_phase| world
            .application
            .select_application_read_observation(&host_phase, &retained)
            .is_ok())
        .expect("installed host call opens"));
    assert!(source_basis_is_admitted(&source, current, true));
    assert!(!source_basis_is_admitted(&source, current, false));

    let foreign = installed_authorization_world(true);
    let selected_foreign = world
        .application
        .with_host_advancement(|host_phase| {
            foreign.selected_product(&host_phase.test_host_request())
        })
        .expect("installed host call opens");
    let foreign_current = selected_foreign.product().observation();
    assert!(!source_basis_is_admitted(&source, foreign_current, true));
    assert!(foreign
        .application
        .with_host_advancement(|host_phase| foreign
            .application
            .select_application_read_observation(&host_phase, &retained)
            .is_err())
        .expect("installed host call opens"));
}
