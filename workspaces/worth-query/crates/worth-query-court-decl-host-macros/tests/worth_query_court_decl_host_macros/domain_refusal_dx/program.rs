//! The stock program: the macro-declared actions plus the `BinCapacity`
//! invariant, installed through a feature definition that wraps them.
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
    primary_graph::*,
    worth_query_application_contribution, worth_query_application_program,
    WorthQueryApplicationHome, WorthQueryHomeConfig,
};

use super::handler::ShipLotHandler;
use super::invariant::{self, BinCapacity};
use super::schema::*;

worth_query_decl::facade::worth_query_authentication_method!(StockMethod);
worth_query_decl::facade::worth_query_authentication_audience!(StockAudience);
worth_query_decl::facade::worth_query_principal_issuer!(StockIssuer);

pub struct StockFeature;
impl ApplicationFeature<StockSchema> for StockFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "domain-refusal.dx.feature.v1";
}

worth_query_application_contribution! {
    pub feature StockActions<StockSchema> for StockFeature => StockRuntime {
        query(ReadLotBinding) { definition: ReadLotQuery::definition(); };
        body_action(ReceiveLotBinding, LotReceived) as ReceiveLotAction {
            operation: receive_lot_operation();
        };
        action ship(ShipLotBinding, LotShipped, ShipLotHandler) as ShipLotAction {};
    }
}

/// The actions' feature plus the invariant: the program macro installs one
/// feature definition, so this one adds the rule's declaration and runtime.
pub struct StockDefinition;
type Actions = StockActions;

impl ApplicationFeatureDefinition<StockSchema> for StockDefinition {
    type Outputs = <Actions as ApplicationFeatureDefinition<StockSchema>>::Outputs;
    fn members(
        schema: ApplicationSchemaDeclarationBuilder<StockSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<StockSchema> {
        invariant::declare(<Actions as ApplicationFeatureDefinition<StockSchema>>::members(schema))
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        <Actions as ApplicationFeatureDefinition<StockSchema>>::feature_specs()
    }
}

impl Registration<StockSchema> for StockDefinition {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<StockSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<StockSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(<Self as ApplicationFeatureDefinition<StockSchema>>::declare_members(builder))
    }
}

impl WorthQueryFeatureDefinition<StockSchema> for StockDefinition {
    type Configuration = StockRuntime;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<StockSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        <Actions as WorthQueryFeatureDefinition<StockSchema>>::contracts(contracts)
    }
    fn configure(
        runtime: StockRuntime,
        setup: &mut WorthQueryApplicationContributionSetup<'_, StockSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        <Actions as WorthQueryFeatureDefinition<StockSchema>>::configure(runtime, setup)?;
        invariant::install(setup)
    }
}

worth_query_application_program! {
    pub program StockProgram for StockSchema {
        identity: "domain-refusal.dx.program.v1",
        rules: ApplicationRuleList<
            ApplicationRuleAt<
                ApplicationLocalRuleRef<StockSchema, StockFeature, BinCapacity>,
                ApplicationCommitBoundary,
            >,
            ApplicationRuleLeaf,
        >,
        contribution: StockContribution {
            identity: "domain-refusal.dx.contribution.v1",
            members: [Records, OwnerBinding],
        },
        feature_definition: StockDefinition,
    }
}

pub type StockHome = WorthQueryApplicationHome<StockSchema, StockProgram>;

/// Seeded lots: 1 `OK` with 40 units, 2 `EXPIRED`, 3 `MISLABELED`.
pub const SEEDED_LOTS: [(u64, &str, u64); 3] =
    [(1, "OK", 40), (2, "EXPIRED", 12), (3, "MISLABELED", 5)];

fn identity(name: &str) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(StockIssuer, name).unwrap()
}

fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<StockSchema>,
    installed: &WorthQueryInstalledApplicationSchema<StockSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let binding = installed
        .principal_binding(OwnerBinding::reference())
        .unwrap();
    for (key, id, name, status) in [
        (
            "owner",
            1_u64,
            "owner",
            WorthQueryPrincipalMappingStatus::Enabled,
        ),
        (
            "revoked",
            2_u64,
            "revoked",
            WorthQueryPrincipalMappingStatus::Disabled,
        ),
    ] {
        let key = WorthQueryApplicationPrincipalKey::new(key).unwrap();
        graph.bind_principal(&binding, key, id, identity(name), status)?;
    }
    for (lot, state, quantity) in SEEDED_LOTS {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Lot::reference(),
                WorthQueryApplicationEntityKey::new(format!("lot-{lot}")).unwrap(),
            )
            .field(LotFacts::lot_id(), lot)
            .field(LotFacts::state(), state.to_owned())
            .field(LotFacts::quantity(), quantity),
        )?;
    }
    Ok(())
}

pub fn open(path: &std::path::Path, clock: &WorthQueryAdmissionClock) -> StockHome {
    let build = ApplicationBuildIdentity::parse("5d0c5d0c-5d0c-4d0c-8d0c-5d0c5d0c5d0c").unwrap();
    WorthQueryApplicationHome::open_directory(
        path,
        WorthQueryHomeConfig::new(build, (StockRuntime::new(ShipLotHandler),))
            .with_clock(clock.clone())
            .with_initial_state(|_, graph, installed| seed(graph, installed)),
    )
    .unwrap_or_else(|denial| panic!("stock Home opens: {denial}"))
}

/// Authenticates `name` ("owner" or the disabled "revoked") as a local process.
pub fn principal(
    home: &StockHome,
    clock: &WorthQueryAdmissionClock,
    name: &str,
) -> WorthQueryAuthenticatedExternalPrincipal<StockSchema> {
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        clock.clone(),
    );
    WorthQueryLocalProcessPrincipal::new(
        identity(name),
        StockAudience,
        StockMethod,
        Duration::from_secs(600),
    )
    .unwrap()
    .admit(home.primary_graph().installed_schema(), clock.clone())
    .expect("local principal admits")
    .authenticate_local(&scope)
    .expect("local principal authenticates")
}
