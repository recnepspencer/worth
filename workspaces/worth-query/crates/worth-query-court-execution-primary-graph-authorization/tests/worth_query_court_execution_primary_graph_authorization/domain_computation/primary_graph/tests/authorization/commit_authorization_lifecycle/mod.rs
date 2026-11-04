use std::cell::Cell;
use std::time::Duration;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use crate::domain_computation::primary_graph::tests::authorization::capability_progression::admitted_capability_operation;
use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;

#[test]
fn commit_authorization_rechecks_cancellation_when_governed() {
    let world = installed_capability_authorization_world();
    world
        .authorization_time
        .script([time(100), time(100), time(100)]);
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(60)),
        cancellation.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let mut admission = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_operation(
                &(phase).test_host_request_for_test(),
                &world,
                &principal,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let commit_authorization = admission
        .take_authorization_dependencies_for_test(
            (*(*world.application.runtime_for_test()).authorization_for_test()).bridge_for_test(),
        )
        .unwrap();
    let commit_lane = (*(*world.application.runtime_for_test()).primary_provider_for_test())
        .application_branch_commit_lane_for_test(
            admission
                .graph_work_for_test()
                .mutation_product()
                .unwrap()
                .observation_for_test(),
        );
    let coordination = commit_lane.enter_for_test();
    let proof = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            commit_authorization
                .authorize_application_commit_for_test(
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
        .govern_for_test((), |()| governed_action_ran.set(true))
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
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let mut source = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_operation(
                &(phase).test_host_request_for_test(),
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
                &(phase).test_host_request_for_test(),
                &world,
                &principal,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let source_authorization = source
        .take_authorization_dependencies_for_test(
            (*(*world.application.runtime_for_test()).authorization_for_test()).bridge_for_test(),
        )
        .unwrap();
    let commit_lane = (*(*world.application.runtime_for_test()).primary_provider_for_test())
        .application_branch_commit_lane_for_test(
            target
                .graph_work_for_test()
                .mutation_product()
                .unwrap()
                .observation_for_test(),
        );
    let coordination = commit_lane.enter_for_test();

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            source_authorization.authorize_application_commit_for_test(
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
            let host_custody = phase.test_host_request_for_test();
            authenticated_principal(&host_custody, &source_world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let mut admission = source_world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            admitted_capability_operation(&host_custody, &source_world, &principal, &request)
        })
        .expect("the fixture read owns its installed host call");
    let commit_authorization = admission
        .take_authorization_dependencies_for_test(
            (*(*source_world.application.runtime_for_test()).authorization_for_test())
                .bridge_for_test(),
        )
        .unwrap();
    let commit_lane = (*(*source_world.application.runtime_for_test()).primary_provider_for_test())
        .application_branch_commit_lane_for_test(
            admission
                .graph_work_for_test()
                .mutation_product()
                .unwrap()
                .observation_for_test(),
        );
    let coordination = commit_lane.enter_for_test();

    let Err(denial) = foreign_world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            commit_authorization.authorize_application_commit_for_test(
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
