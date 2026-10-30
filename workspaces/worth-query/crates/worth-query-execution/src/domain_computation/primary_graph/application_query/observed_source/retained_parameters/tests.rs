//! Real completed-query roster funding and final-owner release.
use super::*;
use crate::domain_computation::primary_graph::{
    tests::fixture::{
        installed_authorization_world, live_scope, status_parameter, AccountStatus,
        TestAccountSourceBinding,
    },
    WorthQueryApplicationQueryAccessContext, WorthQueryPrincipalResolutionMode,
    WorthQueryProductQueryControls,
};
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

#[test]
fn completed_query_roster_keeps_original_funding_until_final_observation_close() {
    let world = installed_authorization_world(true);
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
            let selected = world.selected_product(&phase.test_host_request());
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
                    AccountStatus::reference(),
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
            let roster = &row.parameters.rows;
            let quote = std::alloc::Layout::array::<Arc<WorthQueryObservedSourceMeaning>>(1)
                .unwrap()
                .size() as u64;
            assert_eq!(
                roster.charged_payload_bytes(),
                Some(quote),
                "the actual original request funds the pointer roster"
            );
            assert!(Arc::ptr_eq(&row.source_meaning, &roster[0]));
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
