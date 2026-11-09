//! Genuine fresh capability admissions and a retained governed query observation.
use super::*;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationReadObservation;
type Access = WorthQueryAdmittedApplicationCapabilityAccess<
    Schema,
    Inspect,
    InspectOperation,
    query::model::Read,
>;
pub(super) fn access(
    host: &Host,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    scope: &WorthQueryRequestScope,
) -> Access {
    host.with_host_advancement(|phase| {
        let selected = host
            .runtime()
            .on_branch(host.current_world())
            .select(&phase)
            .unwrap();
        let binding = host
            .installed_schema()
            .principal_binding(Actor::reference())
            .unwrap();
        let principal = selected
            .resolve_authenticated_principal(
                &phase,
                &binding,
                external,
                scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let capability = host
            .installed_schema()
            .capability(Inspect::reference(), InspectOperation::reference())
            .unwrap();
        selected
            .admit_capability_access(
                &phase,
                &principal,
                &capability,
                query::model::Read("a".into()),
                scope,
            )
            .unwrap()
    })
    .unwrap()
}
pub(super) fn navigation_access(
    host: &Host,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAdmittedApplicationCapabilityAccess<
    Schema,
    navigation::Navigate,
    navigation::SelectSnapshot,
    query::model::Read,
> {
    host.with_host_advancement(|phase| {
        let selected = host
            .runtime()
            .on_branch(host.current_world())
            .select(&phase)
            .unwrap();
        let binding = host
            .installed_schema()
            .principal_binding(Actor::reference())
            .unwrap();
        let principal = selected
            .resolve_authenticated_principal(
                &phase,
                &binding,
                external,
                scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let capability = host
            .installed_schema()
            .capability(
                navigation::Navigate::reference(),
                navigation::SelectSnapshot::reference(),
            )
            .unwrap();
        selected
            .admit_capability_access(
                &phase,
                &principal,
                &capability,
                query::model::Read("a".into()),
                scope,
            )
            .unwrap()
    })
    .unwrap()
}
pub(super) fn read(
    host: &Host,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    scope: &WorthQueryRequestScope,
    observation: &WorthQueryApplicationReadObservation,
) -> (WorthQueryApplicationQueryObservation, Vec<u64>) {
    host.with_host_advancement(|phase| {
        let runtime = host.runtime();
        let selected = runtime
            .on_branch(host.current_world())
            .select(&phase)
            .unwrap();
        let binding = host
            .installed_schema()
            .installed_query_binding::<output::Binding>()
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
                ModelKey::at("ModelKey"),
                "a".to_owned(),
                scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let capability = host
            .installed_schema()
            .capability(Inspect::reference(), InspectOperation::reference())
            .unwrap();
        let capability = selected
            .admit_capability_access(
                &phase,
                &principal,
                &capability,
                query::model::Read("a".into()),
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
        (
            result.retain_inspection_observation().unwrap(),
            result.rows()[0].values.clone(),
        )
    })
    .unwrap()
}
