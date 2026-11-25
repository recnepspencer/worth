//! The program installs the targeted recurring rule and seeds the hub, the
//! `home` Inbox and one Task already holding task id `home/taken#1`.
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
    temporal_contribution::WorthQueryRecurringConfig,
    worth_query_application_program, WorthQueryApplicationHome, WorthQueryHomeConfig,
};

use super::schema::*;

pub const TAKEN_TASK_ID: &str = "home/taken#1";

worth_query_decl::facade::worth_query_authentication_method!(LocalProcessMethod);
worth_query_decl::facade::worth_query_authentication_audience!(TargetAudience);
worth_query_decl::facade::worth_query_principal_issuer!(TargetIssuer);

worth_query_application_program! {
    pub program TargetProgram for TargetSchema {
        identity: "recurring.target.dx.program.v1",
        contribution: TargetContribution {
            identity: "recurring.target.dx.contribution.v1",
            members: [Records, OwnerBinding],
            conditionals: TargetConfiguration { recurrence: Recurrence },
        },
        feature: TargetFeature => TargetDefinition / TargetRuntime {
            identity: "recurring.target.dx.feature.v1",
            query(ReadRulesBinding) { definition: ReadRulesQuery::definition(); };
            query(ReadInboxTasksBinding) { definition: ReadInboxTasksQuery::definition(); };
            action schedule(ScheduleRuleBinding, RuleScheduled, ScheduleRuleHandler) as ScheduleRuleAction {};
        }
    }
}

pub type TargetHome = WorthQueryApplicationHome<TargetSchema, TargetProgram>;

fn owner_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(TargetIssuer, "owner").unwrap()
}

fn local_owner() -> WorthQueryLocalProcessPrincipal {
    WorthQueryLocalProcessPrincipal::new(
        owner_identity(),
        TargetAudience,
        LocalProcessMethod,
        Duration::from_secs(600),
    )
    .unwrap()
}

fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<TargetSchema>,
    installed: &WorthQueryInstalledApplicationSchema<TargetSchema>,
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
            WorthQueryApplicationEntityKey::new("inbox:home").unwrap(),
        )
        .field(InboxFacts::inbox_id(), INBOX.to_owned()),
    )?;
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Task::reference(),
            WorthQueryApplicationEntityKey::new("task:taken").unwrap(),
        )
        .field(TaskFacts::task_id(), TAKEN_TASK_ID.to_owned())
        .field(TaskFacts::fire_number(), 0_u64),
    )?;
    Recurrence::seed_hub(graph)
}

pub fn open(path: &std::path::Path, clock: &WorthQueryAdmissionClock) -> TargetHome {
    let build = ApplicationBuildIdentity::parse("7d7d7d7d-7d7d-4d7d-8d7d-7d7d7d7d7d7d").unwrap();
    let configuration = TargetConfiguration {
        feature: TargetRuntime::new(ScheduleRuleHandler),
        recurrence: WorthQueryRecurringConfig::new(local_owner(), clock.clone()),
    };
    WorthQueryApplicationHome::open_directory(
        path,
        WorthQueryHomeConfig::new(build, (configuration,))
            .with_clock(clock.clone())
            .with_initial_state(|_, graph, installed| seed(graph, installed)),
    )
    .unwrap_or_else(|denial| panic!("recurring target Home opens: {denial}"))
}

pub fn owner(
    home: &TargetHome,
    clock: &WorthQueryAdmissionClock,
) -> WorthQueryAuthenticatedExternalPrincipal<TargetSchema> {
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
