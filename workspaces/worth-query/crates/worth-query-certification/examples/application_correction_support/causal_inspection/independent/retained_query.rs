//! Retained governed reads use the actual selected Host query owners.
use super::output_query as query;
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationReadObservation;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryProductQueryControls,
};

pub(in super::super::super) fn read(
    application: &host::OutputHost,
    external: &WorthQueryAuthenticatedExternalPrincipal<NumericSchema>,
    scope: &WorthQueryRequestScope,
    observation: &WorthQueryApplicationReadObservation,
) -> (WorthQueryApplicationQueryObservation, query::Row) {
    let (original, rows) = read_parameters(application, external, scope, observation, true);
    assert_eq!(rows.len(), 1);
    (original, rows[0].clone())
}

pub(in super::super::super) fn read_parameters(
    application: &host::OutputHost,
    external: &WorthQueryAuthenticatedExternalPrincipal<NumericSchema>,
    scope: &WorthQueryRequestScope,
    observation: &WorthQueryApplicationReadObservation,
    enabled: bool,
) -> (WorthQueryApplicationQueryObservation, Vec<query::Row>) {
    application
        .with_host_advancement(|phase| {
            let runtime = application.runtime();
            let selected = runtime
                .on_branch(application.current_world())
                .select(&phase)
                .unwrap();
            let binding = application
                .installed_schema()
                .installed_query_binding::<query::Binding>()
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    binding.principal_binding(),
                    external,
                    scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let entity = selected
                .resolve_entity(
                    &phase,
                    Key::reference(),
                    "numeric".to_owned(),
                    scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let installed = application
                .installed_schema()
                .capability(
                    capability::Inspect::reference(),
                    capability::InspectCausalEvidence::reference(),
                )
                .unwrap();
            let capability = selected
                .admit_capability_access(&phase, &principal, &installed, capability::Input, scope)
                .unwrap();
            let retained = observation.select_on(&phase, runtime).unwrap();
            let plan = selected
                .admit_retained_governed_application_query(
                    &phase,
                    retained,
                    binding.query(),
                    &WorthQueryApplicationQueryAccessContext::new(&principal, &entity),
                    capability,
                    query::parameters(enabled),
                    WorthQueryProductQueryControls::new(binding.limits().maximum_results(), scope),
                )
                .unwrap()
                .retain_projection_for_inspection();
            let result = runtime
                .execute_application_query_one_shot(&phase, plan)
                .unwrap()
                .into_admitted_disclosed()
                .into_output_demand_source();
            (
                result.retain_inspection_observation().unwrap(),
                result.rows().to_vec(),
            )
        })
        .unwrap()
}
