//! Select the Scalar-only query from the actual settled producer observation.
use super::super::super::{authentication::*, causal_inspection::capability, host, schema::*};
use super::query;
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
    },
    application_entry::WorthQueryApplicationReadObservation,
    causal_inspection::*,
    declaration::application_query::*,
    primary_graph::*,
    *,
};

pub(super) fn read(
    application: &host::OutputHost,
    external: &WorthQueryAuthenticatedExternalPrincipal<NumericSchema>,
    scope: &WorthQueryRequestScope,
    observation: &WorthQueryApplicationReadObservation,
) -> (WorthQueryApplicationQueryObservation, Option<u64>) {
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
                .admit_capability_access(
                    &phase,
                    &principal,
                    &installed,
                    capability::Input::at("numeric"),
                    scope,
                )
                .unwrap();
            let retained = observation.select_on(&phase, runtime).unwrap();
            let plan = selected
                .admit_retained_governed_application_query(
                    &phase,
                    retained,
                    binding.query(),
                    &WorthQueryApplicationQueryAccessContext::new(&principal, &entity),
                    capability,
                    ApplicationQueryParameterSet::new(),
                    WorthQueryProductQueryControls::new(binding.limits().maximum_results(), scope),
                )
                .unwrap()
                .retain_projection_for_inspection();
            let result = runtime
                .execute_application_query_one_shot(&phase, plan)
                .unwrap()
                .into_admitted_disclosed()
                .into_output_demand_source();
            assert_eq!(result.rows().len(), 1);
            (
                result.retain_inspection_observation().unwrap(),
                result.rows()[0].total,
            )
        })
        .unwrap()
}
