//! Independent snapshot-identity readback distinguishes retention from retirement.
use super::{schema::*, selection_schema::*};
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Value(pub u64);
worth_query_portable_type!(Value=>"worth.query.retiring.snapshot-value.v1");
worth_query_structured_value_binding!(pub Input for Read {identity:"worth.query.retiring.snapshot-read.v1"});
worth_query_structured_value_binding!(pub ResultBinding for Value {identity:"worth.query.retiring.snapshot-value.v1"});
worth_query_structured_value_binding!(pub Parameters for () {identity:"worth.query.retiring.snapshot-parameters.v1"});
pub struct ValueSlot;
worth_query_portable_type!(ValueSlot=>"worth.query.retiring.snapshot-slot.v1");
worth_query_application_query!(pub Query for Schema,identity "SnapshotQuery",parameters Parameters,result ResultBinding,scope Snapshot=>"Snapshot",name "snapshot");
fn field() -> ApplicationQueryResultFieldRef<
    Query,
    ValueSlot,
    Schema,
    Snapshot,
    SnapshotFacts,
    SnapshotValue,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("value", SnapshotValue::reference())
}
pub fn definition() -> ApplicationQueryDefinition<Schema, Query, (), Value, Snapshot> {
    ApplicationQueryDefinitionBuilder::declare(Query::reference()).root(Snapshot::reference()).scope(Snapshot::reference())
        .result_shape(ApplicationQueryResultShapeBuilder::<Schema,Query,Snapshot,Value,ResultBinding>::new(Snapshot::reference()).field(field()).build())
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public()).basis_support(ApplicationQueryBasisSupport::current_and_pinned()).lanes(ApplicationQueryLaneEligibility::one_shot()).public().build().unwrap()
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
worth_query_query_binding!(pub ReadBinding for Read,schema Schema,
    identity "worth.query.retiring.snapshot-read-binding.v1",input Input,query Query,parameters Parameters=>|_|ApplicationQueryParameterSet::new(),result ResultBinding,
    principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Snapshot,SnapshotFacts,SnapshotIdentity,String,ReadOnly,NoApplicationUnit,field SnapshotIdentity::reference(),value identity,limits results 1);
