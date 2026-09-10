use std::num::NonZeroUsize;
use std::time::{Duration, UNIX_EPOCH};

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

use super::super::super::fixture::{
    admit_touch_account_capability, installed_capability_world_with_label, live_scope,
    AccountIdentity, GovernedHiddenOrderingQuery,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationDisclosed, WorthQueryApplicationProjectionDenialKind,
    WorthQueryApplicationQueryAccessContext, WorthQueryPrincipalResolutionMode,
};

#[test]
fn hidden_ordering_material_is_consumed_before_domain_projection() {
    let world = installed_capability_world_with_label("private");
    world
        .authorization_time
        .script(vec![UNIX_EPOCH + Duration::from_secs(100); 32]);
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
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountIdentity::reference(),
            "account-1".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .application_query(GovernedHiddenOrderingQuery::reference())
        .unwrap();
    let capability = admit_touch_account_capability(&world, &principal, &request).unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .selected_product()
        .admit_governed_application_query(
            &query,
            &access,
            capability,
            ApplicationQueryParameterSet::<GovernedHiddenOrderingQuery>::new(),
            crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
                NonZeroUsize::new(1).unwrap(),
                NonZeroUsize::new(512).unwrap(),
                &request,
            ),
        )
        .unwrap();
    let result = world
        .application
        .execute_application_query_one_shot(plan)
        .unwrap();
    let WorthQueryApplicationDisclosed::Disclosed(activities) = result.rows()[0].activities()
    else {
        panic!("the activity collection must be disclosed");
    };
    let identities = activities
        .iter()
        .map(|activity| match activity.identity() {
            WorthQueryApplicationDisclosed::Disclosed(identity) => identity.as_str(),
            WorthQueryApplicationDisclosed::Omitted(_) => panic!("identity must be disclosed"),
        })
        .collect::<Vec<_>>();
    assert_eq!(identities, ["activity-primary", "activity-secondary"]);
    for activity in activities {
        assert!(matches!(
            activity.sequence(),
            WorthQueryApplicationDisclosed::Omitted(_)
        ));
        assert_eq!(
            activity.required_sequence_denial(),
            WorthQueryApplicationProjectionDenialKind::FieldOmitted
        );
    }
    assert_eq!(result.receipt().ordering_comparison_count(), 1);
    assert_eq!(result.receipt().projected_field_count(), 4);
}
