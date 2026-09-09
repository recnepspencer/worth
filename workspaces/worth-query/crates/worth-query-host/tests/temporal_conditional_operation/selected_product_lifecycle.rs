use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet, primary_graph, runtime,
};

use super::{
    product_query_support::{controls, principal},
    schema::*,
    world::{self, CourtroomWorld},
};

#[test]
fn a_recreated_product_name_cannot_supply_security_to_a_retained_prior_occurrence() {
    let world = CourtroomWorld::publish("ready");
    let request = world::request_scope();
    let principal = principal(&world, &request);
    let query = world
        .application
        .installed_schema()
        .application_query(TemporalIntentQuery::reference())
        .unwrap();
    let source = world.application.admit_current_product_branch().unwrap();
    let identity = create_reused_product(&world, &source);
    let held = world
        .application
        .product_runtime()
        .admit_product_branch(&identity)
        .unwrap();
    let selected = world.application.select_product_branch(&identity).unwrap();
    let scope = selected
        .resolve_entity(
            IntentIdentityField::reference(),
            "intent-1".to_string(),
            &request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let access = primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let plan = selected
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            controls(&request),
        )
        .unwrap();

    let retired = world
        .application
        .product_runtime()
        .retire_product_branch(&held)
        .unwrap();
    assert!(
        retired.owner_retirement_work().is_empty(),
        "the explicit reuse/reuse policy owns no fork to retire"
    );
    let recreated = create_reused_product(&world, &source);
    assert_eq!(
        recreated, identity,
        "World deliberately reuses the descriptive name identity"
    );
    let observer = world.application.application_query_basis_observer();
    let before = observer.observe();
    let denied = world
        .application
        .execute_application_query_one_shot(plan)
        .err()
        .expect("fresh security must refuse the new occurrence");
    assert_eq!(
        denied.kind(),
        primary_graph::WorthQueryApplicationOneShotDenialKind::Authorization(
            primary_graph::WorthQueryOperationAuthorizationDenialKind::ProductSecurityBasis(
                primary_graph::WorthQueryProductBranchAdmissionDenial::IncarnationChanged
            ),
        )
    );
    assert_eq!(
        observer.observe().acquisitions(),
        before.acquisitions(),
        "incarnation denial must precede a security snapshot allocation"
    );
    assert_eq!(observer.observe().active(), 0);
}

fn create_reused_product(
    world: &CourtroomWorld,
    source: &primary_graph::WorthQueryProductBranchLease,
) -> runtime::ProductBranchIdentity {
    let intent = runtime::ProductBranchCreationIntent::from_source(
        "recreated-read",
        runtime::ProductBranchCreationPlans::new(
            runtime::RelationalBranchCreationPlan::ReuseExact,
            runtime::SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .unwrap();
    let outcome = world
        .application
        .product_runtime()
        .create_product_branch(
            source,
            intent,
            &runtime::RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap();
    let runtime::RuntimeWorldBranchCreationOutcome::Performed(observation) = outcome else {
        panic!("reuse/reuse branch creation must perform: {outcome:?}")
    };
    observation.branch_identity().clone()
}
