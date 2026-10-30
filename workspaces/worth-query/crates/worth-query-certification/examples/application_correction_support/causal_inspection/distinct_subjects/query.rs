//! The original observer root and actual nested producer output remain distinct.
use super::super::super::{
    authored_identity_model::AuthoredReference,
    query::{Parameters, ParametersBinding},
    schema::*,
};
use super::super::capability;
use worth_query_decl::facade::worth_query_query_binding;
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};
pub struct RootSlot;
pub struct ChildSlot;
pub struct ValueSlot;
pub struct ReferencesSlot;
worth_query_portable_type!(RootSlot => "distinct-subject.root-slot.v1");
worth_query_portable_type!(ChildSlot => "distinct-subject.child-slot.v1");
worth_query_portable_type!(ValueSlot => "distinct-subject.value-slot.v1");
worth_query_portable_type!(ReferencesSlot => "distinct-subject.references-slot.v1");
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub key: String,
    pub references: Vec<(String, u64)>,
}
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "distinct-subject.result.v1" });
worth_query_structured_value_binding!(pub ChildBinding for () { identity: "distinct-subject.child.v1" });
worth_query_application_query!(pub Query for NumericSchema, identity "distinct-subject.query.v1",
    parameters ParametersBinding, result ResultBinding, scope Number => "Number", name "distinct-subject");
fn root_key() -> ApplicationQueryResultFieldRef<
    Query,
    RootSlot,
    NumericSchema,
    Number,
    NumberFacts,
    Key,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("key", Key::reference())
}
fn child_key() -> ApplicationQueryResultFieldRef<
    Query,
    ChildSlot,
    NumericSchema,
    Number,
    NumberFacts,
    Key,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("child_key", Key::reference())
}
fn calculated() -> ApplicationQueryResultFieldRef<
    Query,
    ValueSlot,
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
fn references() -> ApplicationQueryResultRelationRef<
    Query,
    ReferencesSlot,
    NumericSchema,
    AuthoredReference,
    Number,
    Number,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("references", AuthoredReference::reference())
}
pub fn definition() -> ApplicationQueryDefinition<NumericSchema, Query, Parameters, Row, Number> {
    let child =
        ApplicationQueryResultShapeBuilder::<NumericSchema, Query, Number, (), ChildBinding>::new(
            Number::reference(),
        )
        .field(child_key())
        .field(calculated());
    let shape = ApplicationQueryResultShapeBuilder::<
        NumericSchema,
        Query,
        Number,
        Row,
        ResultBinding,
    >::new(Number::reference())
    .field(root_key())
    .relation(references(), child)
    .build();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "distinct-subject-diagnostic",
        capability::Inspect::reference(),
    )
    .disclose_field_by(
        root_key(),
        capability::text("calculation"),
        ApplicationQueryInfluenceContract::permit_all(),
    )
    .disclose_field_by(
        child_key(),
        capability::text("calculation"),
        ApplicationQueryInfluenceContract::permit_all(),
    )
    .disclose_field_by(
        calculated(),
        capability::text("calculation"),
        ApplicationQueryInfluenceContract::permit_all(),
    )
    .disclose_relation_by(
        references(),
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
        let mut result = Vec::new();
        for child in row.many(references())?.iter() {
            result.push((child.field(child_key())?, child.field(calculated())?));
        }
        Ok(Self {
            key: row.field(root_key())?,
            references: result,
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
worth_query_structured_value_binding!(pub InputBinding for Read { identity: "distinct-subject.input.v1" });
worth_query_query_binding!(pub Binding for Read, schema NumericSchema, identity "distinct-subject.binding.v1",
    input InputBinding, query Query, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(), result ResultBinding,
    principal ActorBinding, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Number, NumberFacts, Key, String, ReadOnly, NoApplicationUnit,
    field Key::reference(), value Read::into_key, limits results 1
);
