//! committed live cause projects with bounded result buffer evidence.
use super::*;

#[test]
fn committed_live_cause_projects_with_bounded_result_buffer_evidence() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let retained_predecessor = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let predecessor_commit = retained_predecessor.product().selected_commit().clone();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
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
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountIdentity::reference(),
                    "account-1".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    let mut lease = world
        .application
        .with_host_advancement(|phase| {
            world
        .selected_product(&(phase).test_host_request(), )
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
            WorthQueryApplicationLiveControls::bounded(request.clone(), 4, 16).unwrap(),
        )
        .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let committed = commit_live_activity(&world, &principal, &request);

    assert_ne!(
        committed.committed_product_publication().composite_commit(),
        &predecessor_commit
    );
    world
        .application
        .with_host_advancement(|phase| {
            retained_predecessor
                .resolve_entity(
                    &phase,
                    AccountIdentity::reference(),
                    "account-1".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .expect("the retained predecessor remains readable after its successor publishes")
        })
        .expect("the fixture read owns its installed host call");

    let WorthQueryApplicationLiveOutcome::Delivered(update) = world
        .application
        .with_host_advancement(|phase| lease.next(&phase, &principal, &request))
        .expect("the fixture read owns its installed host call")
    else {
        panic!("the committed installed live cause must deliver");
    };
    assert_eq!(
        update.product_publication(),
        committed.committed_product_publication()
    );
    let WorthQueryApplicationBasisSelectionIdentity::Product(read_product) =
        update.receipt().basis_identity().selection()
    else {
        panic!("live delivery must retain its exact World product occurrence")
    };
    assert_eq!(
        read_product.branch_identity(),
        update.product_publication().product_branch()
    );
    assert_eq!(
        read_product.lifecycle_incarnation(),
        update.product_publication().product_incarnation()
    );
    assert_eq!(
        read_product.reference_generation(),
        update.product_publication().product_generation()
    );
    assert_eq!(
        read_product.selected_commit(),
        update.product_publication().composite_commit()
    );
    assert_eq!(
        update.receipt().basis_identity().branch_id(),
        &update.product_publication().relational_commit().branch_id
    );
    assert_eq!(update.result().account(), "account-1");
    assert_eq!(
        update.result().activities(),
        &[("activity-primary".to_owned(), 11)]
    );

    let WorthQueryApplicationLiveCloseOutcome::Completed(completion) = lease.close() else {
        panic!("live close must complete its opening graph-read session");
    };
    assert_eq!(completion.release().released_reservation_count(), 1);
}
