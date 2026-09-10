use std::time::Duration;

use super::super::{WorthQueryApplicationLiveControls, WorthQueryApplicationLiveOutcome};
use crate::domain_computation::primary_graph::tests::{
    fixture::{
        installed_authorization_world, live_account_parameters, live_scope, Account,
        AccountIdentity, AccountSummaryParameters, Activity, LiveAccountActivityCause,
        LiveAccountActivityQuery, LiveAccountActivityResult,
    },
    live_delivery_support::commit_live_activity_on_product,
};
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

#[test]
fn sibling_commit_wakes_only_its_exact_product_partition() {
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
    let source = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let intent = worth_runtime_world::facade::ProductBranchCreationIntent::from_source(
        "live-delivery-sibling",
        worth_runtime_world::facade::ProductBranchCreationPlans::new(
            worth_runtime_world::facade::RelationalBranchCreationPlan::ReuseExact,
            worth_runtime_world::facade::SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .unwrap();
    let created = world
        .application
        .product_runtime()
        .create_product_branch(
            &source,
            intent,
            &worth_runtime_world::facade::RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap();
    let worth_runtime_world::facade::RuntimeWorldBranchCreationOutcome::Performed(created) =
        created
    else {
        panic!("the exact sibling must be created")
    };
    let sibling = created.branch_identity().clone();

    let source_account = world
        .selected_product()
        .resolve_entity(
            AccountIdentity::reference(),
            "account-1".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let source_query = world
        .application
        .installed_schema()
        .application_query(LiveAccountActivityQuery::reference())
        .unwrap();
    let mut source_live = world
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
            source_query,
            &principal,
            source_account,
            live_account_parameters("account-1"),
            WorthQueryApplicationLiveControls::bounded(request.clone(), 4, 16, 2_048).unwrap(),
        )
        .unwrap();

    let sibling_account = world
        .application
        .select_product_branch(&sibling)
        .unwrap()
        .resolve_entity(
            AccountIdentity::reference(),
            "account-1".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let sibling_query = world
        .application
        .installed_schema()
        .application_query(LiveAccountActivityQuery::reference())
        .unwrap();
    let mut sibling_live = world
        .application
        .select_product_branch(&sibling)
        .unwrap()
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
            sibling_query,
            &principal,
            sibling_account,
            live_account_parameters("account-1"),
            WorthQueryApplicationLiveControls::bounded(request.clone(), 4, 16, 2_048).unwrap(),
        )
        .unwrap();

    let committed =
        commit_live_activity_on_product(&world, &sibling, &principal, &request, "sibling", 229, 93);
    assert!(matches!(
        source_live.poll(),
        WorthQueryApplicationLiveOutcome::Pending
    ));
    let WorthQueryApplicationLiveOutcome::Delivered(update) = sibling_live.poll() else {
        panic!("the sibling live partition must receive its own commit")
    };
    assert_eq!(
        update.product_publication(),
        committed.committed_product_publication()
    );
    assert_eq!(update.product_publication().product_branch(), &sibling);
    drop(update);
    drop(source_live.close());
    drop(sibling_live.close());
}
