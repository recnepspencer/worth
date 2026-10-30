//! Disclosed original input and actual Calculated output, under one exact binding.
use super::{
    super::{
        query::{Parameters, ParametersBinding},
        schema::*,
    },
    capability,
};
use worth_query_decl::facade::worth_query_query_binding;
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};
pub struct InputSlot;
pub struct OutputSlot;
worth_query_portable_type!(InputSlot => "numeric-correction.causal-input-slot.v1");
worth_query_portable_type!(OutputSlot => "numeric-correction.causal-output-slot.v1");
pub struct OperandSlot;
worth_query_portable_type!(OperandSlot => "numeric-correction.query-operand-slot.v1");
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub operand: Option<u64>,
    pub input: u64,
    pub calculated: u64,
}
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "numeric-correction.causal-row.v1" });
worth_query_application_query!(pub Query for NumericSchema, identity "numeric-correction.causal-query.v1",
    parameters ParametersBinding, result ResultBinding, scope Number => "Number", name "causal-calculation");
fn input() -> ApplicationQueryResultFieldRef<
    Query,
    InputSlot,
    NumericSchema,
    Number,
    NumberFacts,
    Primary,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("input", Primary::reference())
}
fn output() -> ApplicationQueryResultFieldRef<
    Query,
    OutputSlot,
    NumericSchema,
    Number,
    NumberFacts,
    Calculated,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("calculated", Calculated::reference())
}
fn operand() -> ApplicationQueryOptionalResultFieldRef<
    Query,
    OperandSlot,
    NumericSchema,
    Number,
    NumberFacts,
    Operand,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryOptionalResultFieldRef::new("operand", Operand::reference())
}
pub fn definition() -> ApplicationQueryDefinition<NumericSchema, Query, Parameters, Row, Number> {
    let shape = ApplicationQueryResultShapeBuilder::<
        NumericSchema,
        Query,
        Number,
        Row,
        ResultBinding,
    >::new(Number::reference())
    .field(input())
    .field(output())
    .optional_field(operand())
    .build();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "numeric-causal-diagnostic",
        capability::Inspect::reference(),
    )
    .disclose_field_by(
        input(),
        capability::text("calculation"),
        ApplicationQueryInfluenceContract::permit_all(),
    )
    .disclose_optional_field_by(
        operand(),
        capability::text("calculation"),
        ApplicationQueryInfluenceContract::permit_all(),
    )
    .disclose_field_by(
        output(),
        capability::text("calculation"),
        ApplicationQueryInfluenceContract::permit_all(),
    );
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Number::reference())
        .scope(Number::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<NumericSchema, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, NumericSchema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            operand: row.optional_field(operand())?,
            input: row.field(input())?,
            calculated: row.field(output())?,
        })
    }
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Read {
    pub key: String,
}
impl Read {
    fn into_key(self) -> String {
        self.key
    }
}
worth_query_structured_value_binding!(pub InputBinding for Read { identity: "numeric-correction.causal-read-input.v1" });
worth_query_query_binding!(pub Binding for Read, schema NumericSchema, identity "numeric-correction.causal-binding.v1",
    input InputBinding, query Query, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(), result ResultBinding,
    principal ActorBinding, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Number, NumberFacts, Key, String, ReadOnly, NoApplicationUnit,
    field Key::reference(), value Read::into_key, limits results 1
);
