//! The sparse-index program: two declaratively registered stock managed
//! indexes over Optional fields, and Optional-input body actions.
use std::time::Duration;

use worth_query_decl::facade::authentication::*;
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_installation::ApplicationBuildIdentity,
    domain::WorthQueryInstalledApplicationSchema,
    primary_graph::{
        WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
        WorthQueryApplicationPrincipalKey, WorthQueryPrimaryGraphBootstrap,
        WorthQueryPrimaryGraphInstallationDenial,
    },
    worth_query_application_program, WorthQueryApplicationHome, WorthQueryHomeConfig,
};

use super::refinement::{FindExactTitleBinding, FindExactTitleQuery, RefinedTitleIndex};
use super::schema::*;

worth_query_decl::facade::worth_query_authentication_method!(SparseMethod);
worth_query_decl::facade::worth_query_authentication_audience!(SparseAudience);
worth_query_decl::facade::worth_query_principal_issuer!(SparseIssuer);

worth_query_application_program! {
    pub program SparseProgram for SparseSchema {
        identity: "sparse.dx.program.v1",
        contribution: SparseContribution {
            identity: "sparse.dx.contribution.v1",
            members: [Records, OwnerBinding],
        },
        managed_indexes: [TitleIndex, FocusIndex, RefinedTitleIndex],
        feature: SparseFeature => SparseDefinition / SparseRuntime {
            identity: "sparse.dx.feature.v1",
            query(ReadInboxBinding) { definition: ReadInboxQuery::definition(); };
            query(FindTitleBinding) { definition: FindTitleQuery::definition(); };
            query(FindExactTitleBinding) { definition: FindExactTitleQuery::definition(); };
            query(ReadFocusBinding) { definition: ReadFocusQuery::definition(); };
            body_action(AddTaskBinding, TaskAdded) as AddTaskAction { operation: add_task_operation(); };
            body_action(EditTaskBinding, TaskEdited) as EditTaskAction { operation: edit_task_operation(); };
        }
    }
}

pub type SparseHome = WorthQueryApplicationHome<SparseSchema, SparseProgram>;

fn owner_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(SparseIssuer, "owner").unwrap()
}

fn local_owner() -> WorthQueryLocalProcessPrincipal {
    WorthQueryLocalProcessPrincipal::new(
        owner_identity(),
        SparseAudience,
        SparseMethod,
        Duration::from_secs(600),
    )
    .unwrap()
}

fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<SparseSchema>,
    installed: &WorthQueryInstalledApplicationSchema<SparseSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_principal(
        &installed
            .principal_binding(OwnerBinding::reference())
            .unwrap(),
        WorthQueryApplicationPrincipalKey::new("owner").unwrap(),
        1_u64,
        owner_identity(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )?;
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Inbox::reference(),
            WorthQueryApplicationEntityKey::new(INBOX.to_owned()).unwrap(),
        )
        .field(InboxFacts::inbox_id(), INBOX.to_owned()),
    )
}

pub fn open(path: &std::path::Path, clock: &WorthQueryAdmissionClock) -> SparseHome {
    let build = ApplicationBuildIdentity::parse("5a5a5a5a-5a5a-4a5a-8a5a-5a5a5a5a5a5a").unwrap();
    WorthQueryApplicationHome::open_directory(
        path,
        WorthQueryHomeConfig::new(build, (SparseRuntime::new(),))
            .with_clock(clock.clone())
            .with_initial_state(|_, graph, installed| seed(graph, installed)),
    )
    .unwrap_or_else(|denial| panic!("sparse Home opens: {denial}"))
}

pub fn owner(
    home: &SparseHome,
    clock: &WorthQueryAdmissionClock,
) -> WorthQueryAuthenticatedExternalPrincipal<SparseSchema> {
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        clock.clone(),
    );
    local_owner()
        .admit(home.primary_graph().installed_schema(), clock.clone())
        .expect("local owner admits")
        .authenticate_local(&scope)
        .expect("local owner authenticates")
}
