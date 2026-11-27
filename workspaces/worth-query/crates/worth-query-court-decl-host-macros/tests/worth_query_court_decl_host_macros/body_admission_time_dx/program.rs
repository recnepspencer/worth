//! The desk program: the ticket birth body, the noting handler and the
//! `StampedAtAdmission` invariant, opened over a trusted runtime time source.
use std::time::Duration;

use worth_query_decl::facade::application_program::*;
use worth_query_decl::facade::application_schema::fragment::Registration;
use worth_query_decl::facade::application_schema::{
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaDeclarationDenial, CanonicalTimestamp,
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
    primary_graph::*,
    worth_query_application_contribution, worth_query_application_program,
    WorthQueryApplicationHome, WorthQueryHomeConfig,
};

use super::handler::NoteDeskHandler;
use super::invariant::{self, StampedAtAdmission};
use super::schema::*;

worth_query_decl::facade::worth_query_authentication_method!(DeskMethod);
worth_query_decl::facade::worth_query_authentication_audience!(DeskAudience);
worth_query_decl::facade::worth_query_principal_issuer!(DeskIssuer);

pub struct DeskFeature;
impl ApplicationFeature<DeskSchema> for DeskFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "body-admission-time.dx.feature.v1";
}

worth_query_application_contribution! {
    pub feature DeskActions<DeskSchema> for DeskFeature => DeskRuntime {
        query(ReadTicketBinding) { definition: ReadTicketQuery::definition(); };
        query(ReadDeskBinding) { definition: ReadDeskQuery::definition(); };
        body_action(OpenTicketBinding, TicketOpened) as OpenTicketAction {
            operation: open_ticket_operation();
        };
        action note(NoteDeskBinding, DeskNoted, NoteDeskHandler) as NoteDeskAction {};
    }
}

/// The action feature plus the invariant's declaration and runtime.
pub struct DeskDefinition;

impl ApplicationFeatureDefinition<DeskSchema> for DeskDefinition {
    type Outputs = <DeskActions as ApplicationFeatureDefinition<DeskSchema>>::Outputs;
    fn members(
        schema: ApplicationSchemaDeclarationBuilder<DeskSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<DeskSchema> {
        invariant::declare(
            <DeskActions as ApplicationFeatureDefinition<DeskSchema>>::members(schema),
        )
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        <DeskActions as ApplicationFeatureDefinition<DeskSchema>>::feature_specs()
    }
}

impl Registration<DeskSchema> for DeskDefinition {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DeskSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<DeskSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(<Self as ApplicationFeatureDefinition<DeskSchema>>::declare_members(builder))
    }
}

impl WorthQueryFeatureDefinition<DeskSchema> for DeskDefinition {
    type Configuration = DeskRuntime;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<DeskSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        <DeskActions as WorthQueryFeatureDefinition<DeskSchema>>::contracts(contracts)
    }
    fn configure(
        runtime: DeskRuntime,
        setup: &mut WorthQueryApplicationContributionSetup<'_, DeskSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        <DeskActions as WorthQueryFeatureDefinition<DeskSchema>>::configure(runtime, setup)?;
        invariant::install(setup)
    }
}

worth_query_application_program! {
    pub program DeskProgram for DeskSchema {
        identity: "body-admission-time.dx.program.v1",
        rules: ApplicationRuleList<
            ApplicationRuleAt<
                ApplicationLocalRuleRef<DeskSchema, DeskFeature, StampedAtAdmission>,
                ApplicationCommitBoundary,
            >,
            ApplicationRuleLeaf,
        >,
        contribution: DeskContribution {
            identity: "body-admission-time.dx.contribution.v1",
            members: [Records, OwnerBinding],
        },
        feature_definition: DeskDefinition,
    }
}

pub type DeskHome = WorthQueryApplicationHome<DeskSchema, DeskProgram>;

fn identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(DeskIssuer, "owner").unwrap()
}

fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<DeskSchema>,
    installed: &WorthQueryInstalledApplicationSchema<DeskSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let binding = installed
        .principal_binding(OwnerBinding::reference())
        .unwrap();
    let key = WorthQueryApplicationPrincipalKey::new("owner").unwrap();
    let enabled = WorthQueryPrincipalMappingStatus::Enabled;
    graph.bind_principal(&binding, key, 1_u64, identity(), enabled)?;
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Desk::reference(),
            WorthQueryApplicationEntityKey::new("desk-1".to_owned()).unwrap(),
        )
        .field(DeskFacts::desk_id(), 1_u64)
        .field(
            DeskFacts::noted_at(),
            CanonicalTimestamp {
                micros_since_unix_epoch: 0,
            },
        ),
    )
}

fn config(
    trusted_time: impl WorthQueryRuntimeTimeSource,
) -> WorthQueryHomeConfig<'static, DeskSchema, DeskProgram> {
    let build = ApplicationBuildIdentity::parse("de5c0000-0000-4000-8000-de5c00000001").unwrap();
    WorthQueryHomeConfig::new(build, (DeskRuntime::new(NoteDeskHandler),))
        .with_initial_state(|_, graph, installed| seed(graph, installed))
        .with_authorization_time_source(trusted_time)
}

/// Opens an in-memory Home over any trusted runtime time source.
pub fn open(trusted_time: impl WorthQueryRuntimeTimeSource) -> DeskHome {
    WorthQueryApplicationHome::open_memory(config(trusted_time))
        .unwrap_or_else(|denial| panic!("desk Home opens: {denial}"))
}

/// Opens or reopens a Directory Home over a held trusted clock.
pub fn open_directory(
    path: &std::path::Path,
    trusted_time: WorthQueryFixedRuntimeTimeSource,
) -> DeskHome {
    WorthQueryApplicationHome::open_directory(path, config(trusted_time))
        .unwrap_or_else(|denial| panic!("desk Directory Home opens: {denial}"))
}

/// Authenticates the owner as a local process on the system admission clock.
pub fn owner(home: &DeskHome) -> WorthQueryAuthenticatedExternalPrincipal<DeskSchema> {
    let clock = WorthQueryAdmissionClock::system();
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        clock.clone(),
    );
    WorthQueryLocalProcessPrincipal::new(
        identity(),
        DeskAudience,
        DeskMethod,
        Duration::from_secs(600),
    )
    .unwrap()
    .admit(home.primary_graph().installed_schema(), clock)
    .expect("local principal admits")
    .authenticate_local(&scope)
    .expect("local principal authenticates")
}
