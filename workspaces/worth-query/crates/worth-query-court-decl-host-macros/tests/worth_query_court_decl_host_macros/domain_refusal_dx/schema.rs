//! A Stockroom-shaped lot ledger: lots with a state and an on-hand quantity,
//! a body action that receives stock and a hand-written shipping handler.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::{
    application_identity::ApplicationIdentityNamespace,
    application_schema::{fragment::*, *},
    authentication::*,
    worth_query_identity, worth_query_identity_namespace, worth_query_native_scalar_bindings,
    worth_query_principal_binding, worth_query_structured_value_binding, WorthQuery,
    WorthQueryAction, WorthQueryRecord, WorthQueryRelation, WorthQueryRow, WorthQuerySchema,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryInvariantMutationTarget, WorthQueryOperationBodyDenial,
};

pub const STOCK: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("domain-refusal.dx.v1");

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "worth.domain-refusal.dx",
    name = "domain_refusal_dx",
    members(super::program::StockContribution)
)]
pub struct StockSchema;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(fragment, name = "domain_refusal_dx", bindings = Codecs, members(Owner, Mapping, Lot))]
pub struct Records;

worth_query_native_scalar_bindings!(pub Codecs);
impl Scalar<WorthQueryExternalPrincipalIdentity> for Codecs {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Codecs {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(OwnerFacts), reference(StockSchema, Records))]
pub struct Owner;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(MappingFacts),
    relations(MappingOwner),
    reference(StockSchema, Records)
)]
pub struct Mapping;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(LotFacts), reference(StockSchema, Records))]
pub struct Lot;

#[derive(WorthQueryRelation)]
#[worth_query(to = Owner, from = Mapping, reference(StockSchema, Records))]
pub struct MappingOwner;

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = OwnerFacts, fragment = Records, entity = Owner)]
pub struct OwnerRow {
    #[worth_query(equality, read_only)]
    pub owner_id: u64,
}

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = MappingFacts, fragment = Records, entity = Mapping)]
pub struct MappingRow {
    #[worth_query(equality)]
    pub external_identity: WorthQueryExternalPrincipalIdentity,
    #[worth_query(equality)]
    pub mapping_status: WorthQueryPrincipalMappingStatus,
}

/// `state` is `OK`, `EXPIRED` or `MISLABELED`; `quantity` is the units on hand.
#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = LotFacts, fragment = Records, entity = Lot)]
pub struct LotRow {
    #[worth_query(equality, unique, read_only)]
    pub lot_id: u64,
    pub state: String,
    pub quantity: u64,
}

worth_query_structured_value_binding!(pub BodyDenial for WorthQueryOperationBodyDenial {
    identity: worth_query_identity!(STOCK, denial)
});

worth_query_principal_binding! {
    pub OwnerBinding in StockSchema, mapping Mapping => Owner {
        identity: MappingFacts::external_identity(),
        status: MappingFacts::mapping_status(),
        target: MappingOwner,
        principal: OwnerId = OwnerFacts::owner_id(),
    }
    context pub Desktop {
        idempotency: u64 = worth_query_identity!(STOCK, idempotency),
        denial: BodyDenial,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = StockSchema, root = Lot, namespace = STOCK, member)]
pub struct LotView {
    #[worth_query(field = LotId)]
    pub id: u64,
    #[worth_query(field = State)]
    pub state: String,
    #[worth_query(field = Quantity)]
    pub quantity: u64,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = StockSchema, context = Desktop, namespace = STOCK, member,
    result = LotView, root = Lot, scope = LotId, cardinality = ExactlyOne, limits = 1,
)]
pub struct ReadLot {
    #[worth_query(scope)]
    pub lot: u64,
}

/// Sets a lot's on-hand quantity; the bin capacity invariant judges it at commit.
#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = StockSchema, context = Desktop, namespace = STOCK, member,
    input = ReceiveLotInput, operation = ReceiveLotOp, binding = ReceiveLotBinding,
    delivery = LotReceived, body, program, reads(Lot)
)]
pub struct ReceiveLot {
    #[worth_query(scope = LotId)]
    pub lot: u64,
    pub quantity: u64,
}

pub fn receive_lot_operation(
) -> ApplicationOperationDefinition<StockSchema, ReceiveLotOp, ReceiveLot> {
    ReceiveLotOp::reference()
        .typed_body()
        .assign_input(LotFacts::quantity(), ReceiveLot::quantity_input())
        .finish_pure_definition()
        .expect("receive lot body admits")
}

/// Ships units from a lot through a hand-written handler that refuses an
/// expired lot with its own domain reason.
#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = StockSchema, context = Desktop, namespace = STOCK, member,
    input = ShipLotInput, operation = ShipLotOp, binding = ShipLotBinding, delivery = LotShipped,
    result = UnitApplicationValueBinding, decision = ShipDecision, program,
    access(
        read_entities: [Lot];
        read_fields: [LotFacts::lot_id(), LotFacts::state(), LotFacts::quantity()];
        writes: [LotFacts::quantity()];
    )
)]
pub struct ShipLot {
    #[worth_query(scope = LotId)]
    pub lot: u64,
    pub quantity: u64,
}

/// The lot to write and the quantity it keeps after shipping.
pub struct ShipDecision {
    pub target: WorthQueryInvariantMutationTarget<StockSchema, Lot>,
    pub remaining: u64,
}
