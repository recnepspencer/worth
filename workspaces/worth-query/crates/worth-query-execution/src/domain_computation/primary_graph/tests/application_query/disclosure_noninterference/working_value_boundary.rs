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
    WorthQuerySourceExpectationDenialKind,
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
    let query = world
        .application
        .installed_schema()
        .certification_query(GovernedHiddenOrderingQuery::reference())
        .unwrap();
    let capability = world
        .application
        .with_host_advancement(|phase| {
            admit_touch_account_capability(
                &(phase).test_host_request(),
                &world,
                &principal,
                &request,
            )
            .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .admit_governed_application_query(
                    &phase,
                    &query,
                    &access,
                    capability,
                    ApplicationQueryParameterSet::<GovernedHiddenOrderingQuery>::new(),
                    crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
                        NonZeroUsize::new(1).unwrap(),
                        Some(NonZeroUsize::new(512).unwrap()),
                        &request,
                    ),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let result = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
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
    for source in result.observed_sources() {
        let denial = source
            .validate_completeness("GovernedHiddenOrderingQuery")
            .expect_err("a protected internal ordering field cannot yield a complete source");
        assert_eq!(
            denial.kind(),
            WorthQuerySourceExpectationDenialKind::IncompleteFootprint
        );
    }
}
