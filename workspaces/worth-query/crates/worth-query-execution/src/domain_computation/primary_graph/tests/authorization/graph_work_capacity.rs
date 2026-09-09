use std::time::Duration;

use super::super::fixture::{
    installed_authorization_world, live_scope, AccountStatus, TouchAccountOperation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenialKind, WorthQueryPrincipalResolutionMode,
};

#[test]
fn operation_graph_capacity_denial_and_drop_return_exact_reservations() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let authorize = || {
        world.selected_product().authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
    };
    let mut retained = (0..64).map(|_| authorize().unwrap()).collect::<Vec<_>>();

    let denial = match authorize() {
        Err(denial) => denial,
        Ok(_) => panic!("the sixty-fifth operation session must be backpressured"),
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::GraphWorkAdmissionUnavailable
    );
    drop(retained.pop());
    retained.push(authorize().expect("dropping one session returns its provider reservations"));
    drop(retained);
    assert_eq!(world.application.provider_session_resource_count(), 0);
}
