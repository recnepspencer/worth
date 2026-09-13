//! Ordinary installed-operation authorization evidence.

use std::cell::Cell;
use std::time::{Duration, Instant, SystemTime};

use super::super::fixture::{
    installed_authorization_world, live_scope, AccountLabel, AccountStatus, TouchAccountOperation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenialKind, WorthQueryOperationProjectionDenialKind,
    WorthQueryPrincipalResolutionMode,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEncodedScalarValue, OperationExpectsFact, StringApplicationValueBinding,
    TypedMutationPreconditions,
};

impl OperationExpectsFact<TouchAccountOperation> for AccountLabel {}

#[test]
fn current_installed_membership_mints_exact_operation_admission() {
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
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();

    let admitted = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let retried = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &live_scope(),
        )
        .unwrap();

    assert_eq!(admitted.operation(), "TouchAccountOperation");
    assert_eq!(admitted.authorization_requirement_count(), 1);
    assert_ne!(
        admitted.graph_work_session_identity(),
        retried.graph_work_session_identity()
    );
    assert_ne!(
        admitted.graph_work_managed_run_identity(),
        retried.graph_work_managed_run_identity()
    );
    assert_eq!(admitted.graph_work_branch(), retried.graph_work_branch());
    assert_eq!(
        admitted.graph_work_decision_fact_count(),
        admitted.authorization_decision_fact_count()
    );
    assert_eq!(
        admitted.graph_work_principal_entity_id(),
        principal.principal_entity_id()
    );
    assert_eq!(
        admitted.graph_work_scope_entity_id(),
        Some(account.entity_id())
    );
    assert!(admitted.graph_work_capability_identity().is_none());
    assert!(admitted.graph_work_runtime_ordinal() > 0);
    assert!(!admitted.graph_work_provider().is_empty());
    assert_eq!(admitted.allowed_graph_contract(), operation.contracts());
    assert_eq!(admitted.relational_counters().reconstructive_graph_scans, 0);
    assert!(admitted.signal_dependency_count() >= 2);
    assert_eq!(
        admitted.operation_scope_binding(),
        retried.operation_scope_binding()
    );
    let warm_work = admitted.canonical_work().admission();
    assert_eq!(warm_work.basis_preparations(), 0);
    assert_eq!(warm_work.digest_derivations(), 0);
    assert_eq!(warm_work.canonical_entries(), 0);
    assert_eq!(warm_work.canonical_encoded_bytes(), 0);
    assert_eq!(warm_work.canonical_material_allocation_bytes(), 0);
    assert_eq!(warm_work.sha256_input_bytes(), 0);
    assert_eq!(warm_work.sha256_compression_blocks(), 0);
    assert_eq!(warm_work.digest_text_materializations(), 0);
}

#[test]
fn caller_marker_cannot_widen_the_installed_precondition_contract() {
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
    let caller_only = TypedMutationPreconditions::new().expect_fact(
        AccountLabel::reference(),
        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
            "forged-contract-widening".to_owned(),
        )
        .expect("fixture account label must encode"),
    );

    let denial = world
        .selected_product()
        .authorize_operation(&principal, &account, &operation, caller_only, &request)
        .err()
        .expect("caller marker authority must not widen the installed contract");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::MutationPreconditionRejected
    );
}

#[test]
fn missing_membership_and_crossed_runtime_scope_open_no_operation_authority() {
    let denied_world = installed_authorization_world(false);
    let request = live_scope();
    let external = denied_world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = denied_world
        .application
        .select_product_branch(denied_world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &denied_world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = denied_world
        .application
        .select_product_branch(denied_world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let operation = denied_world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let missing_membership = denied_world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .err()
        .expect("missing relationship must deny");
    assert_eq!(
        missing_membership.kind(),
        WorthQueryOperationAuthorizationDenialKind::PermissionDenied
    );

    let foreign = installed_authorization_world(true);
    let foreign_account = foreign
        .application
        .select_product_branch(foreign.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let crossed_scope = denied_world
        .selected_product()
        .authorize_operation(
            &principal,
            &foreign_account,
            &operation,
            Default::default(),
            &request,
        )
        .err()
        .expect("foreign scope must deny");
    assert_eq!(
        crossed_scope.kind(),
        WorthQueryOperationAuthorizationDenialKind::ForeignRuntime
    );
}

#[path = "ordinary_admission/request_lifecycle.rs"]
mod request_lifecycle;
