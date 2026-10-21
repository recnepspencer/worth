//! A genuine row-affine source exposes only its selected parent's revision.
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
worth_query_portable_type!(Value => "worth.query.path-value.revision.v1");
worth_query_structured_value_binding!(pub Input for Read { identity: "worth.query.path-value.read.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Value { identity: "worth.query.path-value.revision.v1" });
worth_query_structured_value_binding!(pub Parameters for () { identity: "worth.query.path-value.parameters.v1" });
pub struct RevisionSlot;
worth_query_portable_type!(RevisionSlot => "worth.query.path-value.revision-slot.v1");
worth_query_application_query!(pub Query for Schema, identity "RevisionQuery", parameters Parameters, result ResultBinding, scope Row => "Row", name "parent_revision");
fn field() -> ApplicationQueryResultFieldRef<
    Query,
    RevisionSlot,
    Schema,
    Parent,
    ParentFacts,
    ParentRevision,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("revision", ParentRevision::reference())
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
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(Row::reference()).reverse(ParentRows::reference()),
        )
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<Schema, Query> for Value {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self(row.field(field())?))
    }
}
fn identity(read: Read) -> String {
    read.0
}
worth_query_query_binding!(pub ReadBinding for Read, schema Schema,
    identity "worth.query.path-value.read-binding.v1", input Input, query Query,
    parameters Parameters => |_| ApplicationQueryParameterSet::new(), result ResultBinding,
    principal PrincipalBinding, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Row, RowFacts, RowIdentity, String, ReadOnly, NoApplicationUnit, field RowIdentity::reference(), value identity, limits results 1);
