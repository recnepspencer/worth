//! Read committed endpoint identities through the native relation, not a receipt or input echo.
use super::super::schema::*;
use super::declaration::SelectedParent;
use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_application_query,
    worth_query_portable_type, worth_query_query_binding, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};
#[derive(Clone)]
pub struct Read(pub String);
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Value(pub String, pub worth_relational::facade::identity::EntityId);
worth_query_structured_value_binding!(pub Input for Read {identity:"worth.query.reference.read.v1"});
worth_query_structured_value_binding!(pub ResultBinding for Value {identity:"worth.query.reference.result.v1"});
worth_query_structured_value_binding!(pub Parameters for () {identity:"worth.query.reference.parameters.v1"});
pub struct Slot;
worth_query_portable_type!(Slot=>"worth.query.reference.slot.v1");
worth_query_application_query!(pub Query for Schema,identity "SelectedParentQuery",parameters Parameters,result ResultBinding,scope Row=>"Row",name "selected_parent");
fn field() -> ApplicationQueryResultFieldRef<
    Query,
    Slot,
    Schema,
    Parent,
    ParentFacts,
    ParentIdentity,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("identity", ParentIdentity::reference())
}
pub fn definition() -> ApplicationQueryDefinition<Schema, Query, (), Value, Row> {
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Parent::reference())
        .scope(Row::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<Schema, Query, Parent, Value, ResultBinding>::new(
                Parent::reference(),
            )
            .field(field())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 12))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(Row::reference()).forward(SelectedParent::reference()),
        )
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<Schema, Query> for Value {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self(row.field(field())?, row.entity_id()))
    }
}
fn row(input: Read) -> String {
    input.0
}
worth_query_query_binding!(pub ReadBinding for Read,schema Schema,identity "worth.query.reference.read-binding.v1",input Input,query Query,parameters Parameters=>|_|ApplicationQueryParameterSet::new(),result ResultBinding,
    principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Row,RowFacts,RowIdentity,String,ReadOnly,NoApplicationUnit,field RowIdentity::reference(),value row,limits results 1);
