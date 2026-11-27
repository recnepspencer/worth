//! The QMS program: the site-status and role-assignment actions plus the two
//! standing role contracts, opened over a held trusted clock and, when given,
//! the Home's capability calendar.
use std::time::Duration;

use worth_query_decl::facade::application_program::*;
use worth_query_decl::facade::application_schema::fragment::Registration;
use worth_query_decl::facade::application_schema::{
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaDeclarationDenial,
};
use worth_query_decl::facade::authentication::*;
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_contribution::{
        WorthQueryApplicationContributionContracts, WorthQueryApplicationContributionSetup,
        WorthQueryFeatureDefinition,
    },
    application_installation::ApplicationBuildIdentity,
    domain::WorthQueryInstalledApplicationSchema,
    local_calendar::WorthQueryTimeZone,
    primary_graph::*,
    worth_query_application_contribution, worth_query_application_program,
    WorthQueryApplicationHome, WorthQueryHomeConfig,
};

use super::assign::AssignRoleHandler;
use super::grants::{self, ACTIVE, APPROVE, PURPOSE, REVIEW};
use super::schema::*;
use super::{Window, SITES};

worth_query_decl::facade::worth_query_authentication_method!(QmsMethod);
worth_query_decl::facade::worth_query_authentication_audience!(QmsAudience);
worth_query_decl::facade::worth_query_principal_issuer!(QmsIssuer);

pub struct QmsFeature;
impl ApplicationFeature<QmsSchema> for QmsFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "standing-grant.dx.feature.v1";
}

worth_query_application_contribution! {
    pub feature QmsActions<QmsSchema> for QmsFeature => QmsRuntime {
        body_action(SetSiteStatusBinding, SiteStatusSet) as SetSiteStatusAction {
            operation: set_site_status_operation();
        };
        action assign(AssignRoleBinding, RoleAssigned, AssignRoleHandler) as AssignRoleAction {};
    }
}

/// The actions' feature plus the two standing role contracts.
pub struct QmsDefinition;
type Actions = QmsActions;

impl ApplicationFeatureDefinition<QmsSchema> for QmsDefinition {
    type Outputs = <Actions as ApplicationFeatureDefinition<QmsSchema>>::Outputs;
    fn members(
        schema: ApplicationSchemaDeclarationBuilder<QmsSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<QmsSchema> {
        grants::declare(<Actions as ApplicationFeatureDefinition<QmsSchema>>::members(schema))
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        <Actions as ApplicationFeatureDefinition<QmsSchema>>::feature_specs()
    }
}

impl Registration<QmsSchema> for QmsDefinition {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<QmsSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<QmsSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(<Self as ApplicationFeatureDefinition<QmsSchema>>::declare_members(builder))
    }
}

impl WorthQueryFeatureDefinition<QmsSchema> for QmsDefinition {
    type Configuration = QmsRuntime;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<QmsSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        <Actions as WorthQueryFeatureDefinition<QmsSchema>>::contracts(contracts)
    }
    fn configure(
        runtime: QmsRuntime,
        setup: &mut WorthQueryApplicationContributionSetup<'_, QmsSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        <Actions as WorthQueryFeatureDefinition<QmsSchema>>::configure(runtime, setup)
    }
}

worth_query_application_program! {
    pub program QmsProgram for QmsSchema {
        identity: "standing-grant.dx.program.v1",
        rules: ApplicationRuleLeaf,
        contribution: QmsContribution {
            identity: "standing-grant.dx.contribution.v1",
            members: [Records, MemberBinding],
        },
        feature_definition: QmsDefinition,
    }
}

pub type QmsHome = WorthQueryApplicationHome<QmsSchema, QmsProgram>;

/// Members: alice (1) holds the seeded roles; bob (2) holds none.
pub const MEMBERS: [(&str, u64); 2] = [("alice", 1), ("bob", 2)];

fn identity(name: &str) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(QmsIssuer, name).unwrap()
}

fn key<Entity>(value: &str) -> WorthQueryApplicationEntityKey<QmsSchema, Entity> {
    WorthQueryApplicationEntityKey::new(value.to_owned()).unwrap()
}

/// One active standing grant of `role` over `site`, held by alice.
fn grant(
    graph: &mut WorthQueryPrimaryGraphBootstrap<QmsSchema>,
    site: &str,
    role: &str,
    window: &Window,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let grant = format!("{site}-{role}");
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(RoleGrant::reference(), key(&grant))
            .field(RoleGrantFacts::grant_id(), grant.clone())
            .field(RoleGrantFacts::role(), role.to_owned())
            .field(RoleGrantFacts::purpose(), PURPOSE.to_owned())
            .field(RoleGrantFacts::grant_status(), ACTIVE.to_owned())
            .field(RoleGrantFacts::valid_from(), window.valid_from)
            .field(RoleGrantFacts::valid_until(), window.valid_until)
            .field(RoleGrantFacts::first_day(), window.first_day)
            .field(RoleGrantFacts::last_day(), window.last_day),
    )?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        RoleHolder::reference(),
        &format!("{grant}-holder"),
        key("alice"),
        key(&grant),
    ))?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        GrantSite::reference(),
        &format!("{grant}-site"),
        key(&grant),
        key(site),
    ))
}

fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<QmsSchema>,
    installed: &WorthQueryInstalledApplicationSchema<QmsSchema>,
    window: &Window,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let binding = installed
        .principal_binding(MemberBinding::reference())
        .unwrap();
    let enabled = WorthQueryPrincipalMappingStatus::Enabled;
    for (name, id) in MEMBERS {
        let principal = WorthQueryApplicationPrincipalKey::new(name).unwrap();
        graph.bind_principal(&binding, principal, id, identity(name), enabled)?;
    }
    for site in SITES {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(Site::reference(), key(site))
                .field(SiteFacts::site_id(), site.to_owned())
                .field(SiteFacts::site_status(), ACTIVE.to_owned()),
        )?;
        for role in [REVIEW, APPROVE] {
            grant(graph, site, role, window)?;
        }
    }
    Ok(())
}

fn config(
    clock: WorthQueryFixedRuntimeTimeSource,
    calendar: Option<WorthQueryTimeZone>,
    window: Window,
) -> WorthQueryHomeConfig<'static, QmsSchema, QmsProgram> {
    let build = ApplicationBuildIdentity::parse("5a1e0000-0000-4000-8000-5a1e00000001").unwrap();
    let config = WorthQueryHomeConfig::new(build, (QmsRuntime::new(AssignRoleHandler),))
        .with_initial_state(move |_, graph, installed| seed(graph, installed, &window))
        .with_authorization_time_source(clock);
    match calendar {
        Some(calendar) => config.with_capability_calendar(calendar),
        None => config,
    }
}

/// Opens (or reopens) the Directory Home at `path`.
pub fn open(
    path: &std::path::Path,
    clock: WorthQueryFixedRuntimeTimeSource,
    calendar: Option<WorthQueryTimeZone>,
    window: Window,
) -> Result<QmsHome, worth_query_host::facade::WorthQueryDirectoryOpenDenial<QmsSchema, QmsProgram>>
{
    WorthQueryApplicationHome::open_directory(path, config(clock, calendar, window))
}

/// `name` authenticated as a local process, with the request scope it acts in.
pub struct Caller {
    pub principal: WorthQueryAuthenticatedExternalPrincipal<QmsSchema>,
    pub scope: WorthQueryRequestScope,
}

pub fn caller(home: &QmsHome, name: &str) -> Caller {
    let clock = WorthQueryAdmissionClock::system();
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        clock.clone(),
    );
    let principal = WorthQueryLocalProcessPrincipal::new(
        identity(name),
        QmsAudience,
        QmsMethod,
        Duration::from_secs(600),
    )
    .unwrap()
    .admit(home.primary_graph().installed_schema(), clock)
    .expect("local principal admits")
    .authenticate_local(&scope)
    .expect("local principal authenticates");
    Caller { principal, scope }
}
