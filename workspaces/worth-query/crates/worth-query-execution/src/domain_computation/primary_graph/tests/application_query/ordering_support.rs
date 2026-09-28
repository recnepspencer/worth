use std::time::Duration;

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

use super::super::fixture::{
    installed_authorization_world, live_scope, status_parameter, AccountStatus,
};
use super::{current_controls, installed_ordered_query};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryPrincipalResolutionMode,
};

#[test]
fn path_bound_ordering_mechanism_opens_exact_plan_authority() {
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
    let query = installed_ordered_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .selected_product()
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new()
                .bind(status_parameter(), "open".to_string())
                .unwrap(),
            current_controls(&request),
        )
        .unwrap();
    assert!(plan
        .graph_read_plan()
        .requirements()
        .requires_kind(
            worth_query_admission::facade::graph_read_access::WorthQueryGraphReadAccessRequirementKind::OrderingSupport,
        ));
    let result = world
        .application
        .execute_application_query_one_shot(plan)
        .unwrap();
    assert_eq!(result.rows()[0].label(), "primary");
}
