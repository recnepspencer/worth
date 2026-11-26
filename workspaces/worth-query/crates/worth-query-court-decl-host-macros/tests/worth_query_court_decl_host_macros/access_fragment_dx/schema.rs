//! A stockroom: bins hold an on-hand quantity and a label, lots carry the
//! quantity a receipt books, and every posting records a movement linked
//! from its bin.
use worth_query_decl::facade::{
    application_identity::ApplicationIdentityNamespace, application_schema::fragment::*,
    authentication::*, worth_query_identity, worth_query_identity_namespace,
    worth_query_native_scalar_bindings, worth_query_principal_binding,
    worth_query_structured_value_binding, WorthQuery, WorthQueryRecord, WorthQueryRelation,
    WorthQueryRow, WorthQuerySchema,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDenial;

pub const STOCK: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("access-fragment.dx.v1");

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "worth.access-fragment.dx",
    name = "access_fragment_dx",
    members(super::program::StockContribution)
)]
pub struct StockSchema;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    fragment,
    name = "access_fragment_dx",
    bindings = Codecs,
    members(Owner, Mapping, Bin, Lot, Movement)
)]
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
#[worth_query(
    entity,
    aspects(BinFacts),
    relations(BinMovements),
    reference(StockSchema, Records)
)]
pub struct Bin;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(LotFacts), reference(StockSchema, Records))]
pub struct Lot;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(MovementFacts), reference(StockSchema, Records))]
pub struct Movement;

#[derive(WorthQueryRelation)]
#[worth_query(to = Owner, from = Mapping, reference(StockSchema, Records))]
pub struct MappingOwner;

#[derive(WorthQueryRelation)]
#[worth_query(to = Movement, from = Bin, reference(StockSchema, Records))]
pub struct BinMovements;

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

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = BinFacts, fragment = Records, entity = Bin)]
pub struct BinRow {
    #[worth_query(equality, unique, read_only)]
    pub bin_id: u64,
    pub on_hand: u64,
    /// Read or written by no posting fragment.
    pub label: String,
}

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = LotFacts, fragment = Records, entity = Lot)]
pub struct LotRow {
    #[worth_query(equality, unique, read_only)]
    pub lot_id: u64,
    pub lot_quantity: u64,
}

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = MovementFacts, fragment = Records, entity = Movement)]
pub struct MovementRow {
    #[worth_query(equality, unique, read_only)]
    pub movement_id: u64,
    pub moved: u64,
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
    context pub Desk {
        idempotency: u64 = worth_query_identity!(STOCK, idempotency),
        denial: BodyDenial,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = StockSchema, root = Bin, namespace = STOCK, member)]
pub struct BinView {
    #[worth_query(field = BinId)]
    pub id: u64,
    #[worth_query(field = OnHand)]
    pub on_hand: u64,
    #[worth_query(field = Label)]
    pub label: String,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = StockSchema, context = Desk, namespace = STOCK, member,
    result = BinView, root = Bin, scope = BinId, cardinality = ExactlyOne, limits = 1,
)]
pub struct ReadBin {
    #[worth_query(scope)]
    pub bin: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = StockSchema, root = Movement, namespace = STOCK, member)]
pub struct MovementView {
    #[worth_query(field = MovementId)]
    pub id: u64,
    #[worth_query(field = Moved)]
    pub moved: u64,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = StockSchema, context = Desk, namespace = STOCK, member,
    result = MovementView, root = Movement, scope = MovementId, limits = 1,
)]
pub struct ReadMovement {
    #[worth_query(scope)]
    pub movement: u64,
}
