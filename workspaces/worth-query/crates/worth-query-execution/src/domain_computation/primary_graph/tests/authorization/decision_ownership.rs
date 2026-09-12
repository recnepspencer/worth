use std::time::Duration;

use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

use super::super::fixture::{
    installed_blocked_authorization_world, live_scope, AccountStatus, TouchAccountOperation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenialKind, WorthQueryPrincipalResolutionMode,
};

#[test]
fn matching_prohibited_path_denies_only_through_bridge_decision_authority() {
    let world = installed_blocked_authorization_world();
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            &external,
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

    let outcome = world.selected_product().authorize_operation(
        &principal,
        &account,
        &operation,
        TypedMutationPreconditions::new(),
        &request,
    );
    let denial = outcome
        .err()
        .expect("matching required and prohibited paths must deny");

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::PermissionDenied
    );
}
