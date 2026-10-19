//! Native traversal observes the committed selection, not operation input or receipt labels.
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
pub struct Value(pub String, pub worth_relational::facade::identity::EntityId);
worth_query_portable_type!(Value=>"worth.query.retiring.current-identity.v2");
worth_query_structured_value_binding!(pub Input for Read {identity:"worth.query.retiring.current-read.v1"});
worth_query_structured_value_binding!(pub ResultBinding for Value {identity:"worth.query.retiring.current-identity.v2"});
worth_query_structured_value_binding!(pub Parameters for () {identity:"worth.query.retiring.current-parameters.v1"});
pub struct IdentitySlot;
worth_query_portable_type!(IdentitySlot=>"worth.query.retiring.current-slot.v1");
worth_query_application_query!(pub Query for Schema,identity "CurrentSnapshotQuery",parameters Parameters,result ResultBinding,scope Row=>"Row",name "current_snapshot");
fn field() -> ApplicationQueryResultFieldRef<
    Query,
    IdentitySlot,
    Schema,
    Snapshot,
    SnapshotFacts,
    SnapshotIdentity,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("identity", SnapshotIdentity::reference())
}
pub fn definition() -> ApplicationQueryDefinition<Schema, Query, (), Value, Row> {
    ApplicationQueryDefinitionBuilder::declare(Query::reference()).root(Snapshot::reference()).scope(Row::reference())
        .result_shape(ApplicationQueryResultShapeBuilder::<Schema,Query,Snapshot,Value,ResultBinding>::new(Snapshot::reference()).field(field()).build())
        .cardinality(ApplicationQueryCardinality::ExactlyOne).dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1,1,12))
        .disclosure(ApplicationQueryDisclosureContract::public()).basis_support(ApplicationQueryBasisSupport::current_and_pinned()).lanes(ApplicationQueryLaneEligibility::one_shot()).public()
        .root_path(ApplicationQueryRootPath::from(Row::reference()).forward(CurrentSnapshot::reference())).build().unwrap()
}
impl WorthQueryApplicationProjection<Schema, Query> for Value {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self(row.field(field())?, row.entity_id()))
    }
}
fn identity(read: Read) -> String {
    read.0
}
worth_query_query_binding!(pub ReadBinding for Read,schema Schema,
    identity "worth.query.retiring.current-read-binding.v1",input Input,query Query,parameters Parameters=>|_|ApplicationQueryParameterSet::new(),result ResultBinding,
    principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Row,RowFacts,RowIdentity,String,ReadOnly,NoApplicationUnit,field RowIdentity::reference(),value identity,limits results 1);
