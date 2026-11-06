//! Original-observation acquisition for exact private post-admission tests.
use worth_query_execution::facade::application_inspection::WorthQueryApplicationQueryObservation;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
use crate::domain_computation::primary_graph::tests::fixture::AccountStatus;
use crate::domain_computation::primary_graph::tests::fixture::current_output_source::AlternateAccountSourceBinding;
use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::current_output_source::TestAccountSourceBinding;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls;
use std::num::NonZeroUsize;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

pub fn observe(
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
            let selected = world.selected_product(&phase.test_host_request_for_test());
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
                    AccountStatus::at("AccountStatus"),
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
