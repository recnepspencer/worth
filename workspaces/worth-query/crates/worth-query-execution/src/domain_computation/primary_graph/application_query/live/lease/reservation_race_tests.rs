use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use super::super::{WorthQueryApplicationLiveControls, WorthQueryApplicationLiveOutcome};
use crate::domain_computation::primary_graph::tests::{
    fixture::{
        installed_authorization_world, live_account_parameters, live_scope, Account,
        AccountIdentity, AccountStatus, AccountSummaryParameters, Activity,
        LiveAccountActivityCause, LiveAccountActivityQuery, LiveAccountActivityResult,
    },
    live_delivery_support::{commit_live_activity, live_activity_program},
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationBasisSelectionIdentity, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationIdempotencyBinding, WorthQueryPrincipalResolutionMode,
};

#[test]
fn lease_opened_after_reservation_receives_the_exact_committed_successor() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .selected_product()
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .application_query(LiveAccountActivityQuery::reference())
        .unwrap();
    let account = world
        .selected_product()
        .resolve_entity(
            AccountIdentity::reference(),
            "account-1".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let (parked_tx, parked_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let release_rx = Arc::new(Mutex::new(release_rx));
    world
        .application
        .primary_provider
        .set_application_commit_causality_reservation_hook(Some(Arc::new(move || {
            parked_tx.send(()).unwrap();
            release_rx.lock().unwrap().recv().unwrap();
        })));

    std::thread::scope(|scope| {
        let committed = scope.spawn(|| commit_live_activity(&world, &principal, &request));
        parked_rx.recv().unwrap();
        let mut lease = world
            .selected_product()
            .open_application_query_live::<
                LiveAccountActivityQuery,
                AccountSummaryParameters,
                LiveAccountActivityResult,
                _,
                _,
                Account,
                Activity,
                LiveAccountActivityCause,
            >(
                query,
                &principal,
                account,
                live_account_parameters("account-1"),
                WorthQueryApplicationLiveControls::bounded(request.clone(), 4, 16, 2_048)
                    .unwrap(),
            )
            .unwrap();
        release_tx.send(()).unwrap();
        let committed = committed.join().unwrap();
        world
            .application
            .primary_provider
            .set_application_commit_causality_reservation_hook(None);

        let WorthQueryApplicationLiveOutcome::Delivered(update) = lease.poll() else {
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
    });
}

#[test]
fn publication_without_a_joining_subscriber_releases_marker_and_observation() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .selected_product()
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
        .selected_product()
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let program = live_activity_program(
        &world,
        world.application.product_runtime().default_branch(),
        &principal,
        &account,
        &request,
        "no-join",
    );
    let retention_before = world
        .application
        .product_runtime()
        .owner
        .inspection_port()
        .retention_snapshot()
        .unwrap();

    let WorthQueryApplicationCommitOutcome::Committed(committed) =
        world.application.compare_and_commit_application(
            program,
            WorthQueryApplicationIdempotencyBinding::new([228; 32], [92; 32]),
        )
    else {
        panic!("the no-subscriber application must commit");
    };
    assert_eq!(
        world
            .application
            .primary_provider
            .active_application_commit_causality_partitions(),
        0
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .retained_application_emission_bytes(),
        0
    );
    let committed_branch = committed
        .committed_product_publication()
        .product_branch()
        .clone();
    drop(committed);
    let retention_after_receipt_release = world
        .application
        .product_runtime()
        .owner
        .inspection_port()
        .retention_snapshot()
        .unwrap();
    assert_eq!(
        retention_after_receipt_release.observations(),
        retention_before.observations()
    );
    assert_eq!(
        &committed_branch,
        world.application.product_runtime().default_branch()
    );
}
