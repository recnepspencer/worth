use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
    },
    declaration::application_query::{ApplicationQueryIntent, ApplicationQueryScopeResolution},
    primary_graph::{
        WorthQueryApplicationOutputDemandSource, WorthQueryApplicationQueryAccessContext,
        WorthQueryOutputDemandDenialKind, WorthQueryPrimaryGraphApplicationRuntime,
        WorthQueryPrincipalResolutionMode, WorthQueryProductQueryControls,
    },
};
use worth_query_topology_entry::{
    PlanarOutputFamily, PlanarQuery, PlanarRead, PlanarReadBinding, PlanarReadResult,
};

use super::super::{authentication, installation};
use crate::ConsumerSchema;

type Application = WorthQueryPrimaryGraphApplicationRuntime<ConsumerSchema>;

mod producer_lifecycle;

pub(super) fn producer_lifecycle_probe(
    foreign_schema: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    producer_lifecycle::live_output_and_stale_head_selection(foreign_schema);
}

fn authenticate(
    application: &Application,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<ConsumerSchema> {
    let adapter = authentication::admit(application.installed_schema());
    authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        scope,
    ))
    .expect("the owner-boundary proof authenticates its principal")
}

fn source(
    application: &Application,
    principal: &WorthQueryAuthenticatedExternalPrincipal<ConsumerSchema>,
    request_scope: &WorthQueryRequestScope,
    body_key: &str,
) -> WorthQueryApplicationOutputDemandSource<PlanarQuery, PlanarReadResult> {
    let intent = PlanarRead {
        body_key: body_key.to_owned(),
    };
    let binding = application
        .installed_schema()
        .installed_query_binding::<PlanarReadBinding<ConsumerSchema>>()
        .expect("the source query binding is installed");
    let selected = application
        .on_branch(application.current_world())
        .select()
        .expect("the current program occurrence is selectable");
    let resolved_principal = selected
        .resolve_authenticated_principal(
            binding.principal_binding(),
            principal,
            request_scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("the principal resolves for the source query");
    let (scope_field, scope_value) = intent
        .clone()
        .into_scope()
        .into_field_parts(resolved_principal.principal_identity());
    let scope = selected
        .resolve_entity(
            scope_field,
            scope_value,
            request_scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("the source occurrence resolves");
    let access = WorthQueryApplicationQueryAccessContext::new(&resolved_principal, &scope);
    let plan = selected
        .admit_application_query(
            binding.query(),
            &access,
            <PlanarRead as ApplicationQueryIntent<ConsumerSchema>>::parameters(&intent),
            WorthQueryProductQueryControls::new(
                binding.limits().maximum_results(),
                binding.limits().maximum_work(),
                request_scope,
            ),
        )
        .expect("the source query admits");
    application
        .execute_application_query_one_shot(plan)
        .expect("the source query executes")
        .into_admitted_disclosed()
        .into_output_demand_source()
}
