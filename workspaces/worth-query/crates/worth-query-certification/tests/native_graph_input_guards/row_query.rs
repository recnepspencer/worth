//! Independent governed readback of row publication.
use super::schema::*;
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
#[derive(Clone, Debug)]
pub struct Value(pub u64);
worth_query_portable_type!(Value => "worth.query.path-input.row_query.value.v1");
worth_query_structured_value_binding!(pub Input for Read { identity: "worth.query.path-input.row_query.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Value { identity: "worth.query.path-input.row_query.value.v1" });
worth_query_structured_value_binding!(pub Parameters for () { identity: "worth.query.path-input.row_query.parameters.v1" });
pub struct ValueSlot;
worth_query_portable_type!(ValueSlot => "worth.query.path-input.row_query.slot.v1");
worth_query_application_query!(pub Query for Schema, identity "RowQuery", parameters Parameters,
    result ResultBinding, scope Row => "Row", name "row_query");
fn value() -> ApplicationQueryResultFieldRef<
    Query,
    ValueSlot,
    Schema,
    Row,
    RowFacts,
    RowValue,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("value", RowValue::reference())
}
pub fn definition() -> ApplicationQueryDefinition<Schema, Query, (), Value, Row> {
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Row::reference())
        .scope(Row::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<Schema, Query, Row, Value, ResultBinding>::new(
                Row::reference(),
            )
            .field(value())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<Schema, Query> for Value {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self(row.field(value())?))
    }
}
fn identity(read: Read) -> String {
    read.0
}
worth_query_query_binding!(pub ReadBinding for Read, schema Schema,
    identity "worth.query.path-input.row_query.binding.v1", input Input, query Query,
    parameters Parameters => |_| ApplicationQueryParameterSet::new(), result ResultBinding,
    principal PrincipalBinding, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Row, RowFacts, RowIdentity, String, ReadOnly, NoApplicationUnit, field RowIdentity::reference(), value identity, limits results 1);
