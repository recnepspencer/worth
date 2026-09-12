//! Exact scope-mismatch explanation evidence.

use std::time::Duration;

use super::super::fixture::{
    installed_authorization_world, live_scope, PrincipalIdentityField, TouchAccountOperation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAuthorizationExplanationCause, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryPrincipalResolutionMode,
};

#[test]
fn explicit_scope_mismatch_preserves_its_exact_explanation_cause() {
    let world = installed_authorization_world(true);
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
    let principal_scope = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            PrincipalIdentityField::reference(),
            1_u64,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();

    let Err(denial) = world.selected_product().authorize_operation(
        &principal,
        &principal_scope,
        &operation,
        Default::default(),
        &request,
    ) else {
        panic!("an ability over Account cannot authorize a Principal scope")
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ScopeMismatch
    );
    assert_eq!(
        denial.explanation_cause(),
        Some(WorthQueryApplicationAuthorizationExplanationCause::ScopeMismatch)
    );
}
