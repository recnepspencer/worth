use std::{num::NonZeroUsize, time::Duration};

use super::WorthQueryApplicationQueryObservation;
use crate::domain_computation::primary_graph::{
    tests::fixture::{
        installed_authorization_world, live_scope, status_parameter, AccountStatus,
        AlternateAccountSourceBinding, IdentityExecutionSchema, TestAccountSourceBinding,
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
                let result_set = result.result_set_observation().clone();
                assert_eq!(result_set.source().original_binding(), binding.as_ref());
                let completed = result.into_admitted_disclosed().into_output_demand_source();
                let observation = completed.retain_inspection_observation().unwrap();
                assert_eq!(observation.source.original_binding(), binding.as_ref());
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
