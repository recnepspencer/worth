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
    let selected = world.selected_product();
    let source =
        WorthQueryProductBranchReadIdentity::from_observation(selected.product().observation());
    let retained = selected.retain_application_read();
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .selected_product()
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("the source owner authenticates");
    commit_live_activity(&world, &principal, &request);
    let current = WorthQueryProductBranchReadIdentity::from_observation(
        world.selected_product().product().observation(),
    );
    assert_ne!(source, current);
    assert!(world
        .application
        .select_application_read_observation(&retained)
        .is_ok());
    assert!(source_basis_is_admitted(&source, &current, true));
    assert!(!source_basis_is_admitted(&source, &current, false));

    let foreign = installed_authorization_world(true);
    let foreign_current = WorthQueryProductBranchReadIdentity::from_observation(
        foreign.selected_product().product().observation(),
    );
    assert!(!source_basis_is_admitted(&source, &foreign_current, true));
    assert!(foreign
        .application
        .select_application_read_observation(&retained)
        .is_err());
}
