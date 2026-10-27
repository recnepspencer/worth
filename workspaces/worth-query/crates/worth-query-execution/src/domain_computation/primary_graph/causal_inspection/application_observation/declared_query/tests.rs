use std::{num::NonZeroUsize, time::Duration};

use super::WorthQueryApplicationQueryObservation;
use crate::domain_computation::primary_graph::{
    tests::fixture::{
        installed_authorization_world, live_scope, status_parameter, AccountStatus,
        AlternateAccountSourceBinding, CrossRootQuery, IdentityExecutionSchema,
        TestAccountSourceBinding,
    },
    WorthQueryApplicationQueryAccessContext, WorthQueryPrincipalResolutionMode,
    WorthQueryProductQueryControls,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryParameterSet,
};

type SourceQuery =
    <TestAccountSourceBinding as ApplicationQueryBinding<IdentityExecutionSchema>>::Query;

#[test]
fn native_query_observations_preserve_exact_binding_without_retaining_a_reader() {
    let world = installed_authorization_world(true);
    let schema = world.application.installed_schema();
    let original = schema
        .installed_query_binding::<TestAccountSourceBinding>()
        .unwrap();
    let alternate = schema
        .installed_query_binding::<AlternateAccountSourceBinding>()
        .unwrap();
    let raw = schema
        .certification_query(SourceQuery::reference())
        .unwrap();
    assert_eq!(original.query().identity(), alternate.query().identity());
    assert_eq!(original.query().identity(), raw.identity());
    let expected = [
        original.query().binding_observation().cloned(),
        alternate.query().binding_observation().cloned(),
        None,
    ];
    assert!(expected[0].is_some());
    assert!(expected[1].is_some());
    assert_ne!(expected[0], expected[1]);
    let queries = [original.into_query(), alternate.into_query(), raw];
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let registry = world.application.application_query_basis_observer();
    let baseline = registry.observe().active();
    let observations = world
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
            let account = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
            let mut observations = Vec::new();
            let mut rows = None;
            for (query, binding) in queries.iter().zip(&expected) {
                let plan = world
                    .selected_product(&phase.test_host_request())
                    .admit_application_query(
                        &phase,
                        query,
                        &access,
                        ApplicationQueryParameterSet::new()
                            .bind(status_parameter(), "open".to_owned())
                            .unwrap(),
                        WorthQueryProductQueryControls::new(NonZeroUsize::MIN, &request),
                    )
                    .unwrap();
                let result = world
                    .application
                    .execute_application_query_one_shot(&phase, plan)
                    .unwrap();
                assert_eq!(result.rows().len(), 1);
                if let Some(rows) = &rows {
                    assert_eq!(result.rows(), rows);
                } else {
                    rows = Some(result.rows().to_vec());
                }
                let row = result.observed_sources()[0].clone().retyped::<()>();
                assert_eq!(row.original_binding(), binding.as_ref());
                assert_eq!(row.original_row_footprints().len(), 1);
                assert!(std::ptr::eq(
                    row.footprint_for_test(),
                    row.original_row_footprints().next().unwrap()
                ));
                let result_set = result.result_set_observation().clone();
                assert_eq!(result_set.source().original_binding(), binding.as_ref());
                assert!(std::ptr::eq(
                    row.footprint_for_test(),
                    result_set
                        .source()
                        .original_row_footprints()
                        .next()
                        .unwrap()
                ));
                assert!(result_set.source().footprint_for_test().aspects.is_empty());
                let completed = result.into_admitted_disclosed().into_output_demand_source();
                let observation = completed.retain_inspection_observation().unwrap();
                assert_eq!(observation.source.original_binding(), binding.as_ref());
                assert!(std::ptr::eq(
                    row.footprint_for_test(),
                    observation.source.original_row_footprints().next().unwrap()
                ));
                observations.push(observation);
            }
            observations
        })
        .unwrap();
    assert_eq!(registry.observe().active(), baseline);
    for observation in observations {
        WorthQueryApplicationQueryObservation::close(observation);
    }
    assert_eq!(registry.observe().active(), baseline);
}

#[test]
fn native_query_observation_preserves_many_and_empty_rosters_separately_from_selection() {
    let world = installed_authorization_world(true);
    let schema = world.application.installed_schema();
    let many = schema
        .certification_query(CrossRootQuery::reference())
        .unwrap();
    let empty = schema
        .installed_query_binding::<TestAccountSourceBinding>()
        .unwrap()
        .into_query();
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let registry = world.application.application_query_basis_observer();
    let baseline = registry.observe().active();
    let (many, empty) = world
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
                    &many,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    WorthQueryProductQueryControls::new(NonZeroUsize::new(2).unwrap(), &request),
                )
                .unwrap();
            let result = world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .unwrap();
            assert_eq!(
                result.rows().len(),
                2,
                "real root paths reach both distinct activities"
            );
            let set = result.result_set_observation().source();
            assert!(set.footprint_for_test().aspects.is_empty());
            assert!(set.footprint_for_test().entities.is_empty());
            assert!(set.footprint_for_test().root_selection.is_some());
            assert_eq!(set.original_row_footprints().len(), 2);
            for (source, footprint) in result
                .observed_sources()
                .iter()
                .zip(set.original_row_footprints())
            {
                assert!(std::ptr::eq(source.footprint_for_test(), footprint));
                assert!(footprint.complete);
                assert!(
                    !footprint.aspects.is_empty(),
                    "the original projection reads are distinct from membership evidence"
                );
            }
            let completed = result.into_admitted_disclosed().into_output_demand_source();
            let many_observation = completed.retain_inspection_observation().unwrap();
            drop(completed);
            let plan = world
                .selected_product(&phase.test_host_request())
                .admit_application_query(
                    &phase,
                    &empty,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "no-match".to_owned())
                        .unwrap(),
                    WorthQueryProductQueryControls::new(NonZeroUsize::MIN, &request),
                )
                .unwrap();
            let result = world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .unwrap();
            assert!(result.rows().is_empty());
            assert!(result.observed_sources().is_empty());
            let completed = result.into_admitted_disclosed().into_output_demand_source();
            let empty_observation = completed.retain_inspection_observation().unwrap();
            (many_observation, empty_observation)
        })
        .unwrap();
    assert_eq!(registry.observe().active(), baseline);
    assert_eq!(many.source.original_row_footprints().len(), 2);
    assert_eq!(empty.source.original_row_footprints().len(), 0);
    assert!(empty.source.original_binding().is_some());
    assert!(
        empty.source.footprint_for_test().root_selection.is_some(),
        "absence still has genuine selection evidence"
    );
    many.close();
    empty.close();
    assert_eq!(registry.observe().active(), baseline);
}
