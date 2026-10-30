//! Original-observation acquisition for exact private post-admission tests.
use super::super::WorthQueryApplicationQueryObservation;
use crate::domain_computation::primary_graph::{
    tests::fixture::{
        status_parameter, AccountStatus, AlternateAccountSourceBinding, AuthorizationWorld,
        IdentityExecutionSchema, TestAccountSourceBinding,
    },
    WorthQueryApplicationQueryAccessContext, WorthQueryPrincipalResolutionMode,
    WorthQueryProductQueryControls,
};
use std::num::NonZeroUsize;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

pub(super) fn observe(
    world: &AuthorizationWorld,
    external: &WorthQueryAuthenticatedExternalPrincipal<IdentityExecutionSchema>,
    request: &WorthQueryRequestScope,
    governed: bool,
) -> WorthQueryApplicationQueryObservation {
    let query = if governed {
        world
            .application
            .installed_schema()
            .installed_query_binding::<AlternateAccountSourceBinding>()
            .unwrap()
            .into_query()
    } else {
        world
            .application
            .installed_schema()
            .installed_query_binding::<TestAccountSourceBinding>()
            .unwrap()
            .into_query()
    };
    world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request());
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    external,
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let scope = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    request,
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
                    WorthQueryProductQueryControls::new(NonZeroUsize::MIN, request),
                )
                .unwrap();
            let completed = world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .unwrap()
                .into_admitted_disclosed()
                .into_output_demand_source();
            assert_eq!(completed.receipt().result_count(), 1);
            completed.retain_inspection_observation().unwrap()
        })
        .unwrap()
}
