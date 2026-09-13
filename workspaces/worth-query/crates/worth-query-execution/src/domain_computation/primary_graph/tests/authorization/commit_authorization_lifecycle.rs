use std::cell::Cell;
use std::time::{Duration, Instant};

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use super::super::application_attempt::authenticated_principal;
use super::super::fixture::{installed_capability_authorization_world, live_scope};
use super::capability_progression::{admitted_capability_operation, time};
use crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind;

#[test]
fn commit_authorization_rechecks_cancellation_when_governed() {
    let world = installed_capability_authorization_world();
    world
        .authorization_time
        .script([time(100), time(100), time(100)]);
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    let principal = authenticated_principal(&world, &request);
    let mut admission = admitted_capability_operation(&world, &principal, &request);
    let commit_authorization = admission
        .take_authorization_dependencies(world.application.authorization.bridge())
        .unwrap();
    let commit_lane = world
        .application
        .primary_provider
        .application_branch_commit_lane(
            admission
                .graph_work()
                .mutation_product()
                .unwrap()
                .observation(),
        );
    let coordination = commit_lane.enter();
    let proof = commit_authorization
        .authorize_application_commit(&world.application, &admission, &coordination)
        .unwrap();

    cancellation.cancel();

    let governed_action_ran = Cell::new(false);
    assert!(proof
        .govern((), |()| governed_action_ran.set(true))
        .is_err());
    assert!(!governed_action_ran.get());
}

#[test]
fn commit_basis_cannot_be_paired_with_a_different_admitted_operation() {
    let world = installed_capability_authorization_world();
    world
        .authorization_time
        .script([time(100), time(100), time(100), time(100)]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let mut source = admitted_capability_operation(&world, &principal, &request);
    let target = admitted_capability_operation(&world, &principal, &request);
    let source_authorization = source
        .take_authorization_dependencies(world.application.authorization.bridge())
        .unwrap();
    let commit_lane = world
        .application
        .primary_provider
        .application_branch_commit_lane(
            target
                .graph_work()
                .mutation_product()
                .unwrap()
                .observation(),
        );
    let coordination = commit_lane.enter();

    let Err(denial) = source_authorization.authorize_application_commit(
        &world.application,
        &target,
        &coordination,
    ) else {
        panic!("a commit basis must remain bound to its originating admission");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::InconsistentDecision
    );
}

#[test]
fn commit_basis_cannot_be_revalidated_by_a_foreign_runtime() {
    let source_world = installed_capability_authorization_world();
    let foreign_world = installed_capability_authorization_world();
    source_world
        .authorization_time
        .script([time(100), time(100)]);
    let request = live_scope();
    let principal = authenticated_principal(&source_world, &request);
    let mut admission = admitted_capability_operation(&source_world, &principal, &request);
    let commit_authorization = admission
        .take_authorization_dependencies(source_world.application.authorization.bridge())
        .unwrap();
    let commit_lane = source_world
        .application
        .primary_provider
        .application_branch_commit_lane(
            admission
                .graph_work()
                .mutation_product()
                .unwrap()
                .observation(),
        );
    let coordination = commit_lane.enter();

    let Err(denial) = commit_authorization.authorize_application_commit(
        &foreign_world.application,
        &admission,
        &coordination,
    ) else {
        panic!("a foreign runtime must not revalidate an admitted operation");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ForeignRuntime
    );
}
