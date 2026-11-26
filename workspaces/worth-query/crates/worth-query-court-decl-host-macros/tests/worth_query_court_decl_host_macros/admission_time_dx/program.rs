//! The dated-lot program: the stamping action plus the `TrustedDates`
//! invariant, opened over a held trusted clock or the default system clock.
use std::time::Duration;

use worth_query_decl::facade::application_program::*;
use worth_query_decl::facade::application_schema::fragment::Registration;
use worth_query_decl::facade::application_schema::{
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaDeclarationDenial, CanonicalDate,
    CanonicalTimestamp,
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

use super::handler::StampLotHandler;
use super::invariant::{self, TrustedDates};
use super::schema::*;
use super::undo::*;

worth_query_decl::facade::worth_query_authentication_method!(DatedMethod);
worth_query_decl::facade::worth_query_authentication_audience!(DatedAudience);
worth_query_decl::facade::worth_query_principal_issuer!(DatedIssuer);

pub struct DatedFeature;
impl ApplicationFeature<DatedSchema> for DatedFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "admission-time.dx.feature.v1";
}

worth_query_application_contribution! {
    pub feature DatedActions<DatedSchema> for DatedFeature => DatedRuntime {
        schema: |schema| { schema.declared_operation(restore_lot_expiry_definition()) };
        query(ReadLotBinding) { definition: ReadLotQuery::definition(); };
        body_action_ref(ExtendLotBinding, LotExtended) as ExtendLotAction {};
        body_action_ref(RestoreLotExpiryBinding, LotExpiryRestored) as RestoreLotExpiryAction {};
        action stamp(StampLotBinding, LotStamped, StampLotHandler) as StampLotAction {};
    }
}

/// The action feature plus the invariant's declaration and runtime.
pub struct DatedDefinition;

impl ApplicationFeatureDefinition<DatedSchema> for DatedDefinition {
    type Outputs = <DatedActions as ApplicationFeatureDefinition<DatedSchema>>::Outputs;
    fn members(
        schema: ApplicationSchemaDeclarationBuilder<DatedSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<DatedSchema> {
        invariant::declare(<DatedActions as ApplicationFeatureDefinition<
            DatedSchema,
        >>::members(schema))
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        <DatedActions as ApplicationFeatureDefinition<DatedSchema>>::feature_specs()
    }
}

impl Registration<DatedSchema> for DatedDefinition {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DatedSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<DatedSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(<Self as ApplicationFeatureDefinition<DatedSchema>>::declare_members(builder))
    }
}

impl WorthQueryFeatureDefinition<DatedSchema> for DatedDefinition {
    type Configuration = DatedRuntime;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<DatedSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        <DatedActions as WorthQueryFeatureDefinition<DatedSchema>>::contracts(contracts)
    }
    fn configure(
        runtime: DatedRuntime,
        setup: &mut WorthQueryApplicationContributionSetup<'_, DatedSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        <DatedActions as WorthQueryFeatureDefinition<DatedSchema>>::configure(runtime, setup)?;
        invariant::install(setup)
    }
}

worth_query_application_program! {
    pub program DatedProgram for DatedSchema {
        identity: "admission-time.dx.program.v1",
        rules: ApplicationRuleList<
            ApplicationRuleAt<
                ApplicationLocalRuleRef<DatedSchema, DatedFeature, TrustedDates>,
                ApplicationCommitBoundary,
            >,
            ApplicationRuleLeaf,
        >,
        contribution: DatedContribution {
            identity: "admission-time.dx.contribution.v1",
            members: [Records, OwnerBinding, CorrectLotExpiry],
        },
        feature_definition: DatedDefinition,
    }
}

pub type DatedHome = WorthQueryApplicationHome<DatedSchema, DatedProgram>;

fn identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(DatedIssuer, "owner").unwrap()
}

fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<DatedSchema>,
    installed: &WorthQueryInstalledApplicationSchema<DatedSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let binding = installed
        .principal_binding(OwnerBinding::reference())
        .unwrap();
    let key = WorthQueryApplicationPrincipalKey::new("owner").unwrap();
    let enabled = WorthQueryPrincipalMappingStatus::Enabled;
    graph.bind_principal(&binding, key, 1_u64, identity(), enabled)?;
    // Seeded far ahead, so the seed passes `TrustedDates` at any real clock.
    let new_year = CanonicalDate::from_ymd(2999, 1, 1).unwrap();
    for lot in [1_u64, 2] {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Lot::reference(),
                WorthQueryApplicationEntityKey::new(format!("lot-{lot}")).unwrap(),
            )
            .field(LotFacts::lot_id(), lot)
            .field(LotFacts::effective_on(), new_year)
            .field(LotFacts::expires_on(), new_year)
            .field(
                LotFacts::stamped_at(),
                CanonicalTimestamp {
                    micros_since_unix_epoch: 0,
                },
            ),
        )?;
    }
    Ok(())
}

fn config(
    trusted_time: Option<impl WorthQueryRuntimeTimeSource>,
) -> WorthQueryHomeConfig<'static, DatedSchema, DatedProgram> {
    let build = ApplicationBuildIdentity::parse("ad0117ed-0000-4000-8000-ad0117ed0000").unwrap();
    let config = WorthQueryHomeConfig::new(build, (DatedRuntime::new(StampLotHandler),))
        .with_initial_state(|_, graph, installed| seed(graph, installed));
    match trusted_time {
        Some(source) => config.with_authorization_time_source(source),
        None => config,
    }
}

/// Opens an in-memory Home; `trusted_time` replaces the default system clock.
pub fn open(trusted_time: Option<WorthQueryFixedRuntimeTimeSource>) -> DatedHome {
    WorthQueryApplicationHome::open_memory(config(trusted_time))
        .unwrap_or_else(|denial| panic!("dated Home opens: {denial}"))
}

/// Opens an in-memory Home over any trusted runtime time source.
pub fn open_with(trusted_time: impl WorthQueryRuntimeTimeSource) -> DatedHome {
    WorthQueryApplicationHome::open_memory(config(Some(trusted_time)))
        .unwrap_or_else(|denial| panic!("dated Home opens: {denial}"))
}

/// Opens or reopens a Directory Home over a held trusted clock.
pub fn open_directory(
    path: &std::path::Path,
    trusted_time: WorthQueryFixedRuntimeTimeSource,
) -> DatedHome {
    WorthQueryApplicationHome::open_directory(path, config(Some(trusted_time)))
        .unwrap_or_else(|denial| panic!("dated Directory Home opens: {denial}"))
}

/// Authenticates the owner as a local process on the system admission clock.
pub fn owner(home: &DatedHome) -> WorthQueryAuthenticatedExternalPrincipal<DatedSchema> {
    let clock = WorthQueryAdmissionClock::system();
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        clock.clone(),
    );
    WorthQueryLocalProcessPrincipal::new(
        identity(),
        DatedAudience,
        DatedMethod,
        Duration::from_secs(600),
    )
    .unwrap()
    .admit(home.primary_graph().installed_schema(), clock)
    .expect("local principal admits")
    .authenticate_local(&scope)
    .expect("local principal authenticates")
}
