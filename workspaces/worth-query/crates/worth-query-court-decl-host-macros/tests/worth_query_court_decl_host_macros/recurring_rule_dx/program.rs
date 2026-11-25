//! The program installs the recurring rule through a declarative `conditionals:` entry.
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
worth_query_decl::facade::worth_query_authentication_audience!(RecurringAudience);
worth_query_decl::facade::worth_query_principal_issuer!(RecurringIssuer);

worth_query_application_program! {
    pub program RecurringProgram for RecurringSchema {
        identity: "recurring.dx.program.v1",
        contribution: RecurringContribution {
            identity: "recurring.dx.contribution.v1",
            members: [Records, OwnerBinding],
            conditionals: RecurringConfiguration { recurrence: Recurrence },
        },
        feature: RecurringFeature => RecurringDefinition / RecurringRuntime {
            identity: "recurring.dx.feature.v1",
            query(ReadRulesBinding) { definition: ReadRulesQuery::definition(); };
            action schedule(ScheduleRuleBinding, RuleScheduled, ScheduleRuleHandler) as ScheduleRuleAction {};
        }
    }
}

pub type RecurringHome = WorthQueryApplicationHome<RecurringSchema, RecurringProgram>;

fn owner_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(RecurringIssuer, "owner").unwrap()
}

fn local_owner() -> WorthQueryLocalProcessPrincipal {
    WorthQueryLocalProcessPrincipal::new(
        owner_identity(),
        RecurringAudience,
        LocalProcessMethod,
        Duration::from_secs(600),
    )
    .unwrap()
}

fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<RecurringSchema>,
    installed: &WorthQueryInstalledApplicationSchema<RecurringSchema>,
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

pub fn open(path: &std::path::Path, clock: &WorthQueryAdmissionClock) -> RecurringHome {
    let build = ApplicationBuildIdentity::parse("7b7b7b7b-7b7b-4b7b-8b7b-7b7b7b7b7b7b").unwrap();
    let configuration = RecurringConfiguration {
        feature: RecurringRuntime::new(ScheduleRuleHandler),
        recurrence: WorthQueryRecurringConfig::new(local_owner(), clock.clone()),
    };
    WorthQueryApplicationHome::open_directory(
        path,
        WorthQueryHomeConfig::new(build, (configuration,))
            .with_clock(clock.clone())
            .with_initial_state(|_, graph, installed| seed(graph, installed)),
    )
    .unwrap_or_else(|denial| panic!("recurring Home opens: {denial}"))
}

pub fn owner(
    home: &RecurringHome,
    clock: &WorthQueryAdmissionClock,
) -> WorthQueryAuthenticatedExternalPrincipal<RecurringSchema> {
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
