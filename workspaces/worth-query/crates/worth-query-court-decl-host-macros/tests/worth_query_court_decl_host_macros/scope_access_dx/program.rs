//! The board program: the stock scope access joins the contribution as members
//! and two actions, and one `worth_query_shared_access!` binds remote access.
use std::time::Duration;

use worth_query_decl::facade::authentication::*;
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_installation::ApplicationBuildIdentity,
    domain::WorthQueryInstalledApplicationSchema,
    primary_graph::{
        WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
        WorthQueryApplicationPrincipalKey, WorthQueryApplicationRelationSeed,
        WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
    },
    shared_access::SharedAccessDenial,
    worth_query_application_program, worth_query_shared_access, WorthQueryApplicationHome,
    WorthQueryHomeConfig,
};

use super::schema::{project_access, *};
use super::shared::*;

worth_query_decl::facade::worth_query_principal_issuer!(pub BoardIssuer);
worth_query_decl::facade::worth_query_authentication_audience!(pub BoardAudience);
worth_query_decl::facade::worth_query_authentication_method!(pub LocalProcess);

worth_query_application_program! {
    pub program BoardProgram for BoardSchema {
        identity: "scope.access.dx.program.v1",
        contribution: BoardContribution {
            identity: "scope.access.dx.contribution.v1",
            members: [Records, OwnerBinding, project_access::ScopeAccessMembers, ProjectRenamedEffect],
        },
        feature: BoardFeature => BoardDefinition / BoardRuntime {
            identity: "scope.access.dx.feature.v1",
            query(SharedProjectReadBinding) { definition: shared_project_definition(); };
            action grant(
                project_access::GrantScopeAccessBinding,
                project_access::ScopeAccessGranted,
                project_access::ScopeGrantHandler
            ) as GrantAction {};
            action revoke(
                project_access::RevokeScopeAccessBinding,
                project_access::ScopeAccessRevoked,
                project_access::ScopeRevokeHandler
            ) as RevokeAction {};
            action rename(RenameProjectBinding, ProjectRenameDelivered, RenameProjectHandler) as RenameAction {};
        }
    }
}

worth_query_shared_access! {
    program BoardProgram for BoardSchema {
        access: project_access,
        payload: SharedParams,
        query: SharedProjectRead = |scope, _| SharedProjectRead(scope.clone()),
        row: SharedProject = |value| value,
        owner: |schema, request_scope| {
            local(OWNER)
                .admit(schema, WorthQueryAdmissionClock::system())
                .map_err(|_| SharedAccessDenial::HostUnavailable)?
                .authenticate_local(request_scope)
                .map_err(|_| SharedAccessDenial::HostUnavailable)
        },
    }
}

pub type BoardHome = WorthQueryApplicationHome<BoardSchema, BoardProgram>;
pub const OWNER: &str = "owner";
pub const ALICE: &str = "alice";
pub const PROJECT: &str = "project-1";

pub fn identity(subject: &str) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(BoardIssuer, subject).unwrap()
}

fn local(subject: &str) -> WorthQueryLocalProcessPrincipal {
    WorthQueryLocalProcessPrincipal::new(
        identity(subject),
        BoardAudience,
        LocalProcess,
        Duration::from_secs(600),
    )
    .unwrap()
}

fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BoardSchema>,
    installed: &WorthQueryInstalledApplicationSchema<BoardSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let binding = installed
        .principal_binding(OwnerBinding::reference())
        .unwrap();
    for (id, subject) in [(1_u64, OWNER), (2, ALICE)] {
        graph.bind_principal(
            &binding,
            WorthQueryApplicationPrincipalKey::new(subject).unwrap(),
            id,
            identity(subject),
            WorthQueryPrincipalMappingStatus::Enabled,
        )?;
    }
    let key = || WorthQueryApplicationEntityKey::new(PROJECT).unwrap();
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(Project::reference(), key())
            .field(ProjectFacts::project_id(), PROJECT.to_owned())
            .field(ProjectFacts::name(), "Launch".to_owned()),
    )?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        ProjectWatchTarget::reference(),
        "watch-self",
        key(),
        key(),
    ))
}

pub fn open(path: &std::path::Path) -> BoardHome {
    let build = ApplicationBuildIdentity::parse("5c5c5c5c-5c5c-4c5c-8c5c-5c5c5c5c5c5c").unwrap();
    let runtime = BoardRuntime::new(
        project_access::ScopeGrantHandler,
        project_access::ScopeRevokeHandler,
        RenameProjectHandler,
    );
    WorthQueryApplicationHome::open_directory(
        path,
        WorthQueryHomeConfig::new(build, (runtime,))
            .with_initial_state(|_, graph, installed| seed(graph, installed)),
    )
    .unwrap_or_else(|denial| panic!("board Home opens: {denial}"))
}

pub fn owner(home: &BoardHome) -> WorthQueryAuthenticatedExternalPrincipal<BoardSchema> {
    let clock = WorthQueryAdmissionClock::system();
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        clock.clone(),
    );
    local(OWNER)
        .admit(home.primary_graph().installed_schema(), clock)
        .expect("local owner admits")
        .authenticate_local(&scope)
        .expect("local owner authenticates")
}
