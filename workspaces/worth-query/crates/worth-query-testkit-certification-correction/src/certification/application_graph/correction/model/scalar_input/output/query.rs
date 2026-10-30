//! Independently governed result with exactly the real Scalar output slot.
use super::super::super::{
    causal_inspection::capability,
    query::{Parameters, ParametersBinding},
    schema::*,
};
use super::super::{ScalarResult, Total};
use worth_query_decl::facade::worth_query_query_binding;
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};
pub struct Slot;
worth_query_portable_type!(Slot => "numeric-scalar.output-slot.v1");
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub total: Option<u64>,
}
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "numeric-scalar.output-row.v1" });
worth_query_application_query!(pub Query for NumericSchema, identity "numeric-scalar.output-query.v1",
    parameters ParametersBinding, result ResultBinding, scope Number => "Number", name "scalar-output");
fn output() -> ApplicationQueryOptionalAspectFieldRef<
    Query,
    Slot,
    NumericSchema,
    Number,
    ScalarResult,
    Total,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryOptionalAspectFieldRef::new("total", Total::reference())
}
pub fn definition() -> ApplicationQueryDefinition<NumericSchema, Query, Parameters, Row, Number> {
    let shape = ApplicationQueryResultShapeBuilder::<
        NumericSchema,
        Query,
        Number,
        Row,
        ResultBinding,
    >::new(Number::reference())
    .optional_aspect_field(output())
    .build();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "numeric-causal-diagnostic",
        capability::Inspect::reference(),
    )
    .disclose_optional_aspect_field_by(
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
            total: row.optional_aspect_field(output())?,
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
worth_query_structured_value_binding!(pub InputBinding for Read { identity: "numeric-scalar.output-input.v1" });
worth_query_query_binding!(pub Binding for Read, schema NumericSchema, identity "numeric-scalar.output-binding.v1",
    input InputBinding, query Query, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(), result ResultBinding,
    principal ActorBinding, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Number, NumberFacts, Key, String, ReadOnly, NoApplicationUnit,
    field Key::reference(), value Read::into_key, limits results 1);
