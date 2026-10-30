//! Ordinary data source for the installed arithmetic producer, separate from inspection.
use super::super::{
    query::{Parameters, ParametersBinding},
    schema::*,
};
use super::{Bias, InputFacts, Sibling};
use worth_query_decl::facade::worth_query_query_binding;
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};
pub struct PrimarySlot;
worth_query_portable_type!(PrimarySlot => "numeric-aspect.producer-primary-slot.v1");
pub struct BiasSlot;
pub struct SiblingSlot;
worth_query_portable_type!(BiasSlot => "numeric-aspect.producer-input-bias-slot.v1");
worth_query_portable_type!(SiblingSlot => "numeric-aspect.producer-input-sibling-slot.v1");
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub bias: Option<u64>,
    pub sibling: Option<u64>,
    pub input: u64,
}
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "numeric-aspect.producer-input-row.v1" });
worth_query_application_query!(pub Query for NumericSchema, identity "numeric-aspect.producer-input-query.v1",
    parameters ParametersBinding, result ResultBinding, scope Number => "Number", name "aspect-producer-input");
fn input() -> ApplicationQueryResultFieldRef<
    Query,
    PrimarySlot,
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
fn bias() -> ApplicationQueryOptionalAspectFieldRef<
    Query,
    BiasSlot,
    NumericSchema,
    Number,
    InputFacts,
    Bias,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryOptionalAspectFieldRef::new("bias", Bias::reference())
}
fn sibling() -> ApplicationQueryOptionalAspectFieldRef<
    Query,
    SiblingSlot,
    NumericSchema,
    Number,
    InputFacts,
    Sibling,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryOptionalAspectFieldRef::new("sibling", Sibling::reference())
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
    .optional_aspect_field(bias())
    .optional_aspect_field(sibling())
    .build();
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Number::reference())
        .scope(Number::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
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
            bias: row.optional_aspect_field(bias())?,
            sibling: row.optional_aspect_field(sibling())?,
            input: row.field(input())?,
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
worth_query_structured_value_binding!(pub InputBinding for Read { identity: "numeric-aspect.producer-input-read-input.v1" });
worth_query_query_binding!(pub Binding for Read, schema NumericSchema, identity "numeric-aspect.producer-input-binding.v1",
    input InputBinding, query Query, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(), result ResultBinding,
    principal ActorBinding, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Number, NumberFacts, Key, String, ReadOnly, NoApplicationUnit,
    field Key::reference(), value Read::into_key, limits results 1
);
