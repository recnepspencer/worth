use std::num::NonZeroUsize;
use std::time::Duration;

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

use super::super::fixture::{
    installed_authorization_world, live_scope, AccountIdentity, AuthorizationWorld,
    OptionalAccountFieldQuery, OptionalAccountFieldResult,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryPrincipalResolutionMode,
};

#[test]
fn optional_result_field_distinguishes_present_value_from_lawful_absence() {
    let world = installed_authorization_world(true);

    let present = execute(&world, "account-1");
    assert_eq!(present.note(), Some("reviewed"));
    assert_eq!(present.score(), None);
    let absent = execute(&world, "account-2");
    assert_eq!(absent.note(), None);
    assert_eq!(absent.score(), None);
}

fn execute(world: &AuthorizationWorld, account: &str) -> OptionalAccountFieldResult {
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountIdentity::reference(),
            account.to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .application_query(OptionalAccountFieldQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let plan = world
        .selected_product()
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::<OptionalAccountFieldQuery>::new(),
            crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
                NonZeroUsize::new(1).unwrap(),
                NonZeroUsize::new(256).unwrap(),
                &request,
            ),
        )
        .unwrap();
    let result = world
        .application
        .execute_application_query_one_shot(plan)
        .unwrap();
    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].account(), account);
    assert_eq!(result.receipt().projected_field_count(), 3);
    assert!(result.receipt().disclosure().omitted().is_empty());
    result.rows()[0].clone()
}
