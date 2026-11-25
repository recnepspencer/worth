//! The program installs the hooked recurring rule through `conditionals:`.
use std::time::Duration;

use worth_query_decl::facade::authentication::*;
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_installation::ApplicationBuildIdentity,
    domain::WorthQueryInstalledApplicationSchema,
    primary_graph::{
        WorthQueryApplicationPrincipalKey, WorthQueryPrimaryGraphBootstrap,
        WorthQueryPrimaryGraphInstallationDenial,
    },
    temporal_contribution::WorthQueryRecurringConfig,
    worth_query_application_program, WorthQueryApplicationHome, WorthQueryHomeConfig,
};

use super::schema::*;

worth_query_decl::facade::worth_query_authentication_method!(LocalProcessMethod);
worth_query_decl::facade::worth_query_authentication_audience!(FireAudience);
worth_query_decl::facade::worth_query_principal_issuer!(FireIssuer);

worth_query_application_program! {
    pub program FireProgram for FireSchema {
        identity: "recurring.fire.dx.program.v1",
        contribution: FireContribution {
            identity: "recurring.fire.dx.contribution.v1",
            members: [Records, OwnerBinding],
            conditionals: FireConfiguration { recurrence: Recurrence },
        },
        feature: FireFeature => FireDefinition / FireRuntime {
            identity: "recurring.fire.dx.feature.v1",
            query(ReadRulesBinding) { definition: ReadRulesQuery::definition(); };
            query(ReadRuleTasksBinding) { definition: ReadRuleTasksQuery::definition(); };
            action schedule(ScheduleRuleBinding, RuleScheduled, ScheduleRuleHandler) as ScheduleRuleAction {};
        }
    }
}

pub type FireHome = WorthQueryApplicationHome<FireSchema, FireProgram>;

fn owner_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(FireIssuer, "owner").unwrap()
}

fn local_owner() -> WorthQueryLocalProcessPrincipal {
    WorthQueryLocalProcessPrincipal::new(
        owner_identity(),
        FireAudience,
        LocalProcessMethod,
        Duration::from_secs(600),
    )
    .unwrap()
}

fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<FireSchema>,
    installed: &WorthQueryInstalledApplicationSchema<FireSchema>,
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
    Recurrence::seed_hub(graph)
}

pub fn open(path: &std::path::Path, clock: &WorthQueryAdmissionClock) -> FireHome {
    let build = ApplicationBuildIdentity::parse("6c6c6c6c-6c6c-4c6c-8c6c-6c6c6c6c6c6c").unwrap();
    let configuration = FireConfiguration {
        feature: FireRuntime::new(ScheduleRuleHandler),
        recurrence: WorthQueryRecurringConfig::new(local_owner(), clock.clone()),
    };
    WorthQueryApplicationHome::open_directory(
        path,
        WorthQueryHomeConfig::new(build, (configuration,))
            .with_clock(clock.clone())
            .with_initial_state(|_, graph, installed| seed(graph, installed)),
    )
    .unwrap_or_else(|denial| panic!("recurring fire Home opens: {denial}"))
}

pub fn owner(
    home: &FireHome,
    clock: &WorthQueryAdmissionClock,
) -> WorthQueryAuthenticatedExternalPrincipal<FireSchema> {
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
