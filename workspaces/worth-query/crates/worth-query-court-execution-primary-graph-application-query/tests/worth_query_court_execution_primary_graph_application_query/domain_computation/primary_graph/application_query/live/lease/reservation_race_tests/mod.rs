use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use worth_query_execution::facade::domain_computation::WorthQueryApplicationLiveControls;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationLiveOutcome;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::live_account_query::live_account_parameters;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Activity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::live_account_query::LiveAccountActivityCause;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::LiveAccountActivityQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::live_account_query::LiveAccountActivityResult;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::live_delivery_support::commit_live_activity;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::live_delivery_support::live_activity_program;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationBasisSelectionIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn lease_opened_after_reservation_receives_the_exact_committed_successor() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = world
        .application
        .installed_schema()
        .certification_query(LiveAccountActivityQuery::reference())
        .unwrap();
    let account = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .resolve_entity(
                    &phase,
                    AccountIdentity::at("AccountIdentity"),
                    "account-1".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let (parked_tx, parked_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let release_rx = Arc::new(Mutex::new(release_rx));
    (*(*world.application.runtime_for_test()).primary_provider_for_test())
        .set_application_commit_causality_reservation_hook_for_test(Some(Arc::new(move || {
            parked_tx.send(()).unwrap();
            release_rx.lock().unwrap().recv().unwrap();
        })));

    world
        .application
        .with_host_advancement(|phase| {
            std::thread::scope(|scope| {
                let committed = scope.spawn(|| commit_live_activity(&world, &principal, &request));
                parked_rx.recv().unwrap();
                let mut lease = world
            .selected_product(&(phase).test_host_request_for_test(), )
            .open_application_query_live::<
                LiveAccountActivityQuery,
                AccountSummaryParameters,
                LiveAccountActivityResult,
                _,
                _,
                Account,
                Activity,
                LiveAccountActivityCause,
            >(&phase,
                query,
                &principal,
                account,
                live_account_parameters("account-1"),
                WorthQueryApplicationLiveControls::bounded(request.clone(), 4, 16)
                    .unwrap(),
            )
            .unwrap();
                release_tx.send(()).unwrap();
                let committed = committed.join().unwrap();
                (*(*world.application.runtime_for_test()).primary_provider_for_test())
                    .set_application_commit_causality_reservation_hook_for_test(None);

                let WorthQueryApplicationLiveOutcome::Delivered(update) =
                    lease.next(&phase, &principal, &request)
                else {
                    panic!("the lease linearized before World movement must receive its successor");
                };
                assert_eq!(
                    update.product_publication(),
                    committed.committed_product_publication()
                );
                let WorthQueryApplicationBasisSelectionIdentity::Product(product) =
                    update.receipt().basis_identity().selection()
                else {
                    panic!("live delivery must project the carried product occurrence");
                };
                assert_eq!(
                    product.selected_commit(),
                    committed.committed_product_publication().composite_commit()
                );
            })
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn publication_without_a_joining_subscriber_releases_marker_and_observation() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .resolve_entity(
                    &phase,
                    AccountStatus::at("AccountStatus"),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let retention_before = world
        .application
        .product_runtime()
        .inspection_port_for_test()
        .retention_snapshot()
        .unwrap();
    let program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            live_activity_program(
                &(phase).test_host_request_for_test(),
                &host_custody.borrowed_request_for_test(),
                &world,
                world
                    .application
                    .product_runtime()
                    .default_branch_for_test(),
                &principal,
                &account,
                &request,
                "no-join",
            )
        })
        .expect("the fixture read owns its installed host call");

    let WorthQueryApplicationCommitOutcome::Committed(committed) =
        world.application.compare_and_commit_application(
            program,
            WorthQueryApplicationIdempotencyBinding::new_for_test([228; 32], [92; 32]),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the no-subscriber application must commit");
    };
    assert_eq!(
        (*(*world.application.runtime_for_test()).primary_provider_for_test())
            .active_application_commit_causality_partitions_for_test(),
        0
    );
    // Compare before preparing the live attempt and after it finishes, with
    // the descriptive receipt still retained. Neither receipt owner pins it.
    let retention_after_publication = world
        .application
        .product_runtime()
        .inspection_port_for_test()
        .retention_snapshot()
        .unwrap();
    assert_eq!(
        retention_after_publication.observations(),
        retention_before.observations()
    );
    assert_eq!(
        committed.committed_product_publication().product_branch(),
        world
            .application
            .product_runtime()
            .default_branch_for_test()
    );
}
