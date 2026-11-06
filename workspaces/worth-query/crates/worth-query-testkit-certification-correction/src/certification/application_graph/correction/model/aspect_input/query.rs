//! Disclosed original input and actual Calculated output, under one exact binding.
use super::super::{
    causal_inspection::capability,
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
pub struct InputSlot;
pub struct OutputSlot;
worth_query_portable_type!(InputSlot => "numeric-aspect.causal-input-slot.v1");
worth_query_portable_type!(OutputSlot => "numeric-aspect.causal-output-slot.v1");
pub struct BiasSlot;
pub struct SiblingSlot;
worth_query_portable_type!(BiasSlot => "numeric-aspect.query-bias-slot.v1");
worth_query_portable_type!(SiblingSlot => "numeric-aspect.query-sibling-slot.v1");
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub bias: Option<u64>,
    pub sibling: Option<u64>,
    pub input: u64,
    pub calculated: u64,
}
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "numeric-aspect.causal-row.v1" });
worth_query_application_query!(pub Query for NumericSchema, identity "numeric-aspect.causal-query.v1",
    parameters ParametersBinding, result ResultBinding, scope Number => "Number", name "aspect-input-calculation");
fn input() -> ApplicationQueryResultFieldRef<Query, InputSlot, NumericSchema, Primary> {
    ApplicationQueryResultFieldRef::new("input", Primary::at("Primary"))
}
fn output() -> ApplicationQueryResultFieldRef<Query, OutputSlot, NumericSchema, Calculated> {
    ApplicationQueryResultFieldRef::new("calculated", Calculated::at("Calculated"))
}
fn bias() -> ApplicationQueryOptionalAspectFieldRef<Query, BiasSlot, NumericSchema, Bias> {
    ApplicationQueryOptionalAspectFieldRef::new("bias", Bias::at("Bias"))
}
fn sibling() -> ApplicationQueryOptionalAspectFieldRef<Query, SiblingSlot, NumericSchema, Sibling> {
    ApplicationQueryOptionalAspectFieldRef::new("sibling", Sibling::at("Sibling"))
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
    .optional_aspect_field(bias())
    .optional_aspect_field(sibling())
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
    .disclose_optional_aspect_field_by(
        bias(),
        capability::text("calculation"),
        ApplicationQueryInfluenceContract::permit_all(),
    )
    .disclose_optional_aspect_field_by(
        sibling(),
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
            bias: row.optional_aspect_field(bias())?,
            sibling: row.optional_aspect_field(sibling())?,
            input: row.field(input())?.0,
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
worth_query_structured_value_binding!(pub InputBinding for Read { identity: "numeric-aspect.causal-read-input.v1" });
worth_query_query_binding!(pub Binding for Read, schema NumericSchema, identity "numeric-aspect.causal-binding.v1",
    input InputBinding, query Query, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(), result ResultBinding,
    principal ActorBinding, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Key,
    field Key::at("Key"), value Read::into_key, limits results 1
);
