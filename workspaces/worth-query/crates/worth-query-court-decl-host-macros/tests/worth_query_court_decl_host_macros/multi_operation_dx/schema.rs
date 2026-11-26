//! A counter board: counters keyed by `CounterCounterId` (an entity-prefixed
//! column, as Stockroom's `OrderOrderId`) with a shared, non-unique `label`.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::{
    application_identity::ApplicationIdentityNamespace,
    application_schema::{fragment::*, *},
    authentication::*,
    worth_query_identity, worth_query_identity_namespace, worth_query_native_scalar_bindings,
    worth_query_principal_binding, worth_query_structured_value_binding, WorthQuery,
    WorthQueryAction, WorthQueryRecord, WorthQueryRelation, WorthQueryRow, WorthQuerySchema,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDenial;

pub const BOARD: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("multi-operation.dx.v1");

/// The counter every `ResetMain` writes, whatever its input.
pub const MAIN_COUNTER: u64 = 1;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "worth.multi-operation.dx",
    name = "multi_operation_dx",
    members(super::program::BoardContribution)
)]
pub struct BoardSchema;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(fragment, name = "multi_operation_dx", bindings = Codecs, members(Owner, Mapping, Counter))]
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
#[worth_query(entity, aspects(OwnerFacts), reference(BoardSchema, Records))]
pub struct Owner;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(MappingFacts),
    relations(MappingOwner),
    reference(BoardSchema, Records)
)]
pub struct Mapping;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(CounterFacts), reference(BoardSchema, Records))]
pub struct Counter;

#[derive(WorthQueryRelation)]
#[worth_query(to = Owner, from = Mapping, reference(BoardSchema, Records))]
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

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = CounterFacts, fragment = Records, entity = Counter)]
pub struct CounterRow {
    #[worth_query(column = "CounterCounterId", equality, unique, read_only)]
    pub counter_id: u64,
    /// Shared by several counters, so it cannot select one.
    #[worth_query(equality)]
    pub label: String,
    pub value: u64,
}

worth_query_structured_value_binding!(pub BodyDenial for WorthQueryOperationBodyDenial {
    identity: worth_query_identity!(BOARD, denial)
});

worth_query_principal_binding! {
    pub OwnerBinding in BoardSchema, mapping Mapping => Owner {
        identity: MappingFacts::external_identity(),
        status: MappingFacts::mapping_status(),
        target: MappingOwner,
        principal: OwnerId = OwnerFacts::owner_id(),
    }
    context pub Desk {
        idempotency: u64 = worth_query_identity!(BOARD, idempotency),
        denial: BodyDenial,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = BoardSchema, root = Counter, namespace = BOARD, member)]
pub struct CounterView {
    #[worth_query(field = CounterCounterId)]
    pub id: u64,
    #[worth_query(field = Label)]
    pub label: String,
    #[worth_query(field = Value)]
    pub value: u64,
}

/// Scoped on the counter's own unique key: admitted.
#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = BoardSchema, context = Desk, namespace = BOARD, member,
    result = CounterView, root = Counter, scope = CounterCounterId,
    cardinality = ExactlyOne, limits = 1,
)]
pub struct ReadCounter {
    #[worth_query(scope)]
    pub counter: u64,
}

/// Scoped on the counter's own mutable, non-unique `label`: refused at `build()`.
#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = BoardSchema, context = Desk, namespace = BOARD, member,
    result = CounterView, root = Counter, scope = Label, limits = 8,
)]
pub struct CountersByLabel {
    #[worth_query(scope)]
    pub label: String,
}

/// A fixed-duty action: it always writes the main counter, so its scope is a
/// constant rather than an input field or a dummy scope function.
#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = BoardSchema, context = Desk, namespace = BOARD, member,
    input = ResetMainInput, operation = ResetMainOp, binding = ResetMainBinding,
    delivery = MainReset, body, program, reads(Counter),
    scope = CounterCounterId, scope_const = MAIN_COUNTER,
)]
pub struct ResetMain {
    pub value: u64,
}

pub fn reset_main_operation() -> ApplicationOperationDefinition<BoardSchema, ResetMainOp, ResetMain>
{
    ResetMainOp::reference()
        .typed_body()
        .assign_input(CounterFacts::value(), ResetMain::value_input())
        .finish_pure_definition()
        .expect("reset body admits")
}

/// Selects its counter by the shared `label`; two counters share `"pair"`.
#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = BoardSchema, context = Desk, namespace = BOARD, member,
    input = SetByLabelInput, operation = SetByLabelOp, binding = SetByLabelBinding,
    delivery = LabelSet, body, program, reads(Counter),
)]
pub struct SetByLabel {
    #[worth_query(scope = Label)]
    pub label: String,
    pub value: u64,
}

pub fn set_by_label_operation(
) -> ApplicationOperationDefinition<BoardSchema, SetByLabelOp, SetByLabel> {
    SetByLabelOp::reference()
        .typed_body()
        .assign_input(CounterFacts::value(), SetByLabel::value_input())
        .finish_pure_definition()
        .expect("set-by-label body admits")
}
