use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
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
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let mut admission = world
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
    let proof = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            commit_authorization
                .authorize_application_commit(
                    &host_custody.borrowed_request_for_test(),
                    &world.application,
                    &admission,
                    &coordination,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

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
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let mut source = world
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
    let target = world
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

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            source_authorization.authorize_application_commit(
                &host_custody.borrowed_request_for_test(),
                &world.application,
                &target,
                &coordination,
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("a commit basis must remain bound to its originating admission");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::InconsistentDecision
        )
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
    let principal = source_world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&host_custody, &source_world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let mut admission = source_world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_capability_operation(&host_custody, &source_world, &principal, &request)
        })
        .expect("the fixture read owns its installed host call");
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

    let Err(denial) = foreign_world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            commit_authorization.authorize_application_commit(
                &host_custody.borrowed_request_for_test(),
                &foreign_world.application,
                &admission,
                &coordination,
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("a foreign runtime must not revalidate an admitted operation");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignRuntime)
    );
}
