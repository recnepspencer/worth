use std::num::NonZeroUsize;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_access_fixture::admit_touch_account_capability;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_world_with_label;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::GovernedHiddenOrderingQuery;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationDisclosed;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_execution::facade::domain_computation::WorthQuerySourceExpectationDenialKind;

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
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .select_product_branch_for_test(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch_for_test(),
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
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .select_product_branch_for_test(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch_for_test(),
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
                &(phase).test_host_request_for_test(),
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
                .selected_product(&(phase).test_host_request_for_test())
                .admit_governed_application_query(
                    &phase,
                    &query,
                    &access,
                    capability,
                    ApplicationQueryParameterSet::<GovernedHiddenOrderingQuery>::new(),
                    worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(
                        NonZeroUsize::new(1).unwrap(),
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
            .validate_completeness_for_test("GovernedHiddenOrderingQuery")
            .expect_err("a protected internal ordering field cannot yield a complete source");
        assert_eq!(
            denial.kind(),
            WorthQuerySourceExpectationDenialKind::IncompleteFootprint
        );
    }
}
