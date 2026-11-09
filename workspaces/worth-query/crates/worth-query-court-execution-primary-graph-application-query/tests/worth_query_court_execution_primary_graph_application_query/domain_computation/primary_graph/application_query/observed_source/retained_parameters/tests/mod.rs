//! Real completed-query roster funding and final-owner release.
use std::sync::Arc;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaning;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::current_output_source::TestAccountSourceBinding;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

#[test]
fn completed_query_roster_keeps_original_funding_until_final_observation_close() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let query = world
        .application
        .installed_schema()
        .installed_query_binding::<TestAccountSourceBinding>()
        .unwrap()
        .into_query();
    let request = live_scope();
    let external = world.authenticate("alice", std::time::Duration::from_secs(60), &request);
    let registry = world.application.application_query_basis_observer();
    let baseline = registry.observe().active();
    let (observation, sibling, row_weak) = world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request_for_test());
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let scope = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::at("AccountStatus"),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
            let plan = selected
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_owned())
                        .unwrap(),
                    WorthQueryProductQueryControls::new(std::num::NonZeroUsize::MIN, &request),
                )
                .unwrap();
            let result = world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .unwrap();
            assert_eq!(result.rows().len(), 1);
            let row = &result.observed_sources()[0];
            let roster = (*row.parameters_for_test()).rows_for_test();
            let quote = std::alloc::Layout::array::<Arc<WorthQueryObservedSourceMeaning>>(1)
                .unwrap()
                .size() as u64;
            assert_eq!(
                roster.charged_payload_bytes(),
                Some(quote),
                "the actual original request funds the pointer roster"
            );
            assert!(Arc::ptr_eq(row.source_meaning_for_test(), &roster[0]));
            let row_weak = Arc::downgrade(&roster[0]);
            let completed = result.into_admitted_disclosed().into_output_demand_source();
            let observation = completed.retain_inspection_observation().unwrap();
            let sibling = observation.clone();
            drop(completed);
            assert!(
                row_weak.upgrade().is_some(),
                "opaque evidence retains the genuine meaning after business rows drop"
            );
            (observation, sibling, row_weak)
        })
        .unwrap();
    assert_eq!(registry.observe().active(), baseline);
    observation.close();
    assert!(row_weak.upgrade().is_some());
    sibling.close();
    assert!(
        row_weak.upgrade().is_none(),
        "no source/capsule cycle survives the last observation"
    );
    assert_eq!(registry.observe().active(), baseline);
}
