#[path = "world/authentication.rs"]
mod authentication;
#[path = "world/baseline_graph.rs"]
mod baseline_graph;
#[path = "world/installation.rs"]
mod installation;
#[path = "world/scenario.rs"]
mod scenario;

use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
use worth_query_execution::facade::primary_graph::{
    WorthQueryOperationAuthorizationDenial, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode,
};
use worth_query_installation::facade::WorthQueryInstalledPrincipalBinding;

use super::declaration::{
    ExternalMapping, Principal, PublicationAuthorizationSchema, PublicationCapability,
    PublicationIdentityBinding, PublicationInput, PublicationOperation,
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum CompositionScenario {
    MissingAuthorization,
    ExplicitDeny,
    AccumulatedProhibitions,
}

type InstalledBinding = WorthQueryInstalledPrincipalBinding<
    PublicationAuthorizationSchema,
    PublicationIdentityBinding,
    ExternalMapping,
    Principal,
    u64,
    U64ApplicationValueBinding,
>;

pub(super) struct InstalledWorld {
    pub(super) runtime: WorthQueryPrimaryGraphApplicationRuntime<PublicationAuthorizationSchema>,
    pub(super) binding: InstalledBinding,
}

pub(crate) fn real_denial(scenario: CompositionScenario) -> WorthQueryOperationAuthorizationDenial {
    let world = installation::install_world(scenario);
    let request = authentication::request_scope();
    let external =
        authentication::authenticate_external(world.runtime.installed_schema(), &request);
    let selected = world
        .runtime
        .with_application_advancement(&request, |phase| {
            world
                .runtime
                .on_branch(world.runtime.current_world())
                .select(&phase)
        })
        .expect("installed host call opens")
        .expect("the published product branch must remain admitted");
    let principal = world
        .runtime
        .with_application_advancement(&request, |phase| {
            selected.resolve_authenticated_principal(
                &phase,
                &world.binding,
                &external,
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
        })
        .expect("installed host call opens")
        .unwrap();
    let capability = world
        .runtime
        .installed_schema()
        .capability(
            PublicationCapability::reference(),
            PublicationOperation::reference(),
        )
        .unwrap();

    world
        .runtime
        .with_application_advancement(&request, |phase| {
            selected.admit_capability_access(
                &phase,
                &principal,
                &capability,
                PublicationInput,
                &request,
            )
        })
        .expect("installed host call opens")
        .err()
        .unwrap_or_else(|| panic!("{scenario:?} must deny at real capability admission"))
}
