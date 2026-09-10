#[path = "world/authentication.rs"]
mod authentication;
#[path = "world/baseline_graph.rs"]
mod baseline_graph;
#[path = "world/installation.rs"]
mod installation;
#[path = "world/scenario.rs"]
mod scenario;

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
        .on_branch(world.runtime.current_world())
        .select()
        .expect("the published product branch must remain admitted");
    let principal = selected
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let capability = world
        .runtime
        .installed_schema()
        .capability(
            PublicationCapability::reference(),
            PublicationOperation::reference(),
        )
        .unwrap();

    selected
        .admit_capability_access(&principal, &capability, PublicationInput, &request)
        .err()
        .unwrap_or_else(|| panic!("{scenario:?} must deny at real capability admission"))
}
