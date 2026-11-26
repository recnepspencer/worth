//! One feature carrying the three posting actions and the two reads, seeded
//! with two bins and one lot.
use std::time::Duration;

use worth_query_decl::facade::application_program::*;
use worth_query_decl::facade::authentication::*;
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_installation::ApplicationBuildIdentity,
    domain::WorthQueryInstalledApplicationSchema, primary_graph::*,
    worth_query_application_contribution, worth_query_application_program,
    WorthQueryApplicationHome, WorthQueryHomeConfig,
};

use super::moves::{AdjustBinding, Adjusted, Adjuster};
use super::receipts::{ReceiveBinding, Received, Receiver};
use super::schema::*;
use super::transfers::{ShipBinding, Shipped, Shipper};

worth_query_decl::facade::worth_query_authentication_method!(StockMethod);
worth_query_decl::facade::worth_query_authentication_audience!(StockAudience);
worth_query_decl::facade::worth_query_principal_issuer!(StockIssuer);

pub struct PostingFeature;
impl ApplicationFeature<StockSchema> for PostingFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "access-fragment.dx.posting.v1";
}

worth_query_application_contribution! {
    pub feature PostingActions<StockSchema> for PostingFeature => PostingRuntime {
        action adjust(AdjustBinding, Adjusted, Adjuster) {};
        action receive(ReceiveBinding, Received, Receiver) {};
        action ship(ShipBinding, Shipped, Shipper) {};
        query(ReadBinBinding) { definition: ReadBinQuery::definition(); };
        query(ReadMovementBinding) { definition: ReadMovementQuery::definition(); };
    }
}

worth_query_application_program! {
    pub program StockProgram for StockSchema {
        identity: "access-fragment.dx.program.v1",
        rules: ApplicationRuleLeaf,
        contribution: StockContribution {
            identity: "access-fragment.dx.contribution.v1",
            members: [Records, OwnerBinding],
        },
        feature_definition: PostingActions,
    }
}

pub type StockHome = WorthQueryApplicationHome<StockSchema, StockProgram>;

/// Seeded bins `(id, on_hand)`, each labelled `"seeded"`.
pub const BINS: [(u64, u64); 2] = [(1, 10), (2, 0)];
/// The seeded lot `(id, quantity)`.
pub const LOT: (u64, u64) = (7, 5);

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
    let key = WorthQueryApplicationPrincipalKey::new("owner").unwrap();
    let enabled = WorthQueryPrincipalMappingStatus::Enabled;
    graph.bind_principal(&binding, key, 1_u64, identity("owner"), enabled)?;
    for (id, on_hand) in BINS {
        let key = WorthQueryApplicationEntityKey::new(format!("bin-{id}")).unwrap();
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(Bin::reference(), key)
                .field(BinFacts::bin_id(), id)
                .field(BinFacts::on_hand(), on_hand)
                .field(BinFacts::label(), "seeded".to_owned()),
        )?;
    }
    let (lot, quantity) = LOT;
    let key = WorthQueryApplicationEntityKey::new(format!("lot-{lot}")).unwrap();
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(Lot::reference(), key)
            .field(LotFacts::lot_id(), lot)
            .field(LotFacts::lot_quantity(), quantity),
    )?;
    Ok(())
}

/// Opens the stockroom in memory.
pub fn open(clock: &WorthQueryAdmissionClock) -> StockHome {
    let build = ApplicationBuildIdentity::parse("8e0f8e0f-8e0f-4e0f-8e0f-8e0f8e0f8e0f").unwrap();
    let runtime = PostingRuntime::builder()
        .adjust(Adjuster)
        .receive(Receiver)
        .ship(Shipper)
        .build()
        .expect("every posting action has its handler");
    let config = WorthQueryHomeConfig::new(build, (runtime,))
        .with_clock(clock.clone())
        .with_initial_state(|_, graph, installed| seed(graph, installed));
    WorthQueryApplicationHome::open_memory(config)
        .unwrap_or_else(|denial| panic!("stockroom Home opens: {denial}"))
}

pub fn owner(
    home: &StockHome,
    clock: &WorthQueryAdmissionClock,
) -> WorthQueryAuthenticatedExternalPrincipal<StockSchema> {
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        clock.clone(),
    );
    WorthQueryLocalProcessPrincipal::new(
        identity("owner"),
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
