#[path = "world/authentication.rs"]
pub mod authentication;
#[path = "world/baseline_graph.rs"]
pub mod baseline_graph;
#[path = "world/installation.rs"]
pub mod installation;
#[path = "world/scenario.rs"]
pub mod scenario;

pub use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
pub use worth_query_execution::facade::primary_graph::{
    WorthQueryOperationAuthorizationDenial, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode,
};
pub use worth_query_installation::facade::WorthQueryInstalledPrincipalBinding;

pub use super::declaration::{
    ExternalMapping, Principal, PublicationAuthorizationSchema, PublicationCapability,
    PublicationIdentityBinding, PublicationInput, PublicationOperation,
};

#[derive(Clone, Copy, Debug)]
pub enum CompositionScenario {
    MissingAuthorization,
    ExplicitDeny,
    AccumulatedProhibitions,
}

pub type InstalledBinding = WorthQueryInstalledPrincipalBinding<
    PublicationAuthorizationSchema,
    PublicationIdentityBinding,
    ExternalMapping,
    Principal,
    u64,
    U64ApplicationValueBinding,
>;

pub struct InstalledWorld {
    pub runtime: WorthQueryPrimaryGraphApplicationRuntime<PublicationAuthorizationSchema>,
    pub binding: InstalledBinding,
}

pub fn real_denial(scenario: CompositionScenario) -> WorthQueryOperationAuthorizationDenial {
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
