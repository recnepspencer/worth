//! Independently governed Calculated-only observation of the real settled output.
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
pub struct OutputSlot;
pub struct EnabledParameter;
fn enabled_parameter(
) -> ApplicationQueryParameterRef<Query, EnabledParameter, BoolApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("expected_read_enabled")
}
pub(super) fn parameters(enabled: bool) -> ApplicationQueryParameterSet<Query> {
    ApplicationQueryParameterSet::new()
        .bind(enabled_parameter(), enabled)
        .unwrap()
}
worth_query_portable_type!(OutputSlot => "numeric-correction.causal-output-output-slot.v1");
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub calculated: u64,
}
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "numeric-correction.causal-output-row.v1" });
worth_query_application_query!(pub Query for NumericSchema, identity "numeric-correction.causal-output-query.v1",
    parameters ParametersBinding, result ResultBinding, scope Number => "Number", name "causal-output");
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
pub fn definition() -> ApplicationQueryDefinition<NumericSchema, Query, Parameters, Row, Number> {
    let shape = ApplicationQueryResultShapeBuilder::<
        NumericSchema,
        Query,
        Number,
        Row,
        ResultBinding,
    >::new(Number::reference())
    .field(output())
    .build();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "numeric-causal-diagnostic",
        capability::Inspect::reference(),
    )
    .use_field_by(
        ReadEnabled::reference(),
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
        .parameter(enabled_parameter())
        .where_equal(ReadEnabled::reference(), enabled_parameter())
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<NumericSchema, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, NumericSchema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            calculated: row.field(output())?,
        })
    }
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Read {
    pub key: String,
    pub enabled: bool,
}
impl Read {
    fn into_key(self) -> String {
        self.key
    }
}
worth_query_structured_value_binding!(pub InputBinding for Read { identity: "numeric-correction.causal-output-read-input.v1" });
worth_query_query_binding!(pub Binding for Read, schema NumericSchema, identity "numeric-correction.causal-output-binding.v1",
    input InputBinding, query Query, parameters ParametersBinding => |input: &Read| parameters(input.enabled), result ResultBinding,
    principal ActorBinding, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Number, NumberFacts, Key, String, ReadOnly, NoApplicationUnit,
    field Key::reference(), value Read::into_key, limits results 1
);
