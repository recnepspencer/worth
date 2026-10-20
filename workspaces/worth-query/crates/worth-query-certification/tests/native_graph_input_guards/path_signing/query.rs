//! Independent native readback of both immutable evidence kinds.
use super::{super::schema::*, schema::*};
use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_application_query,
    worth_query_portable_type, worth_query_query_binding, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};
#[derive(Clone)]
pub struct ReadGraph(pub String);
#[derive(Clone)]
pub struct ReadScope(pub String);
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphValue {
    pub value: u64,
    pub revision: u64,
    pub actor: u64,
    pub company: Option<String>,
    pub provenance: ApplicationSigningGraphSubjectProvenance,
}
#[derive(Clone, Debug)]
pub struct ScopeValue(pub u64);
worth_query_structured_value_binding!(pub GraphInput for ReadGraph {identity:"worth.query.signed-path.read-graph.v1"});
worth_query_structured_value_binding!(pub ScopeInput for ReadScope {identity:"worth.query.signed-path.read-scope.v1"});
worth_query_structured_value_binding!(pub GraphResult for GraphValue {identity:"worth.query.signed-path.graph-value.v1"});
worth_query_structured_value_binding!(pub ScopeResult for ScopeValue {identity:"worth.query.signed-path.scope-value.v1"});
worth_query_structured_value_binding!(pub Parameters for () {identity:"worth.query.signed-path.parameters.v1"});
worth_query_application_query!(pub GraphQuery for Schema,identity "SignedPathGraphQuery",parameters Parameters,result GraphResult,scope Row=>"Row",name "graph_evidence");
worth_query_application_query!(pub ScopeQuery for Schema,identity "SignedPathScopeQuery",parameters Parameters,result ScopeResult,scope Row=>"Row",name "scope_evidence");
macro_rules! field {
    ($slot:ident,$name:ident,$query:ty,$entity:ty,$aspect:ty,$field:ty,$value:ty,$identity:literal)=>{
        pub struct $slot;worth_query_portable_type!($slot=>$identity);
        fn $name()->ApplicationQueryResultFieldRef<$query,$slot,Schema,$entity,$aspect,$field,$value,ReadOnly,EqualityPredicate,NoApplicationUnit>{ApplicationQueryResultFieldRef::new(stringify!($name),<$field>::reference())}
    };
}
field!(
    ValueSlot,
    value,
    GraphQuery,
    GraphEvidence,
    GraphFacts,
    SignedValue,
    u64,
    "worth.query.signed-path.value-slot.v1"
);
field!(
    RevisionSlot,
    revision,
    GraphQuery,
    GraphEvidence,
    GraphFacts,
    SignedRevision,
    u64,
    "worth.query.signed-path.revision-slot.v1"
);
field!(
    ActorSlot,
    actor,
    GraphQuery,
    GraphEvidence,
    GraphFacts,
    GraphActor,
    u64,
    "worth.query.signed-path.actor-slot.v1"
);
field!(
    ProvenanceSlot,
    provenance,
    GraphQuery,
    GraphEvidence,
    GraphFacts,
    GraphProvenance,
    ApplicationSigningGraphSubjectProvenance,
    "worth.query.signed-path.provenance-slot.v1"
);
field!(
    ScopeActorSlot,
    scope_actor,
    ScopeQuery,
    ScopeEvidence,
    ScopeFacts,
    ScopeActor,
    u64,
    "worth.query.signed-path.scope-actor-slot.v1"
);
pub struct CompanySlot;
worth_query_portable_type!(CompanySlot=>"worth.query.signed-path.company-slot.v1");
fn company() -> ApplicationQueryOptionalResultFieldRef<
    GraphQuery,
    CompanySlot,
    Schema,
    GraphEvidence,
    GraphFacts,
    SignedCompany,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryOptionalResultFieldRef::new("company", SignedCompany::reference())
}
impl WorthQueryApplicationProjection<Schema, GraphQuery> for GraphValue {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, GraphQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            value: row.field(value())?,
            revision: row.field(revision())?,
            actor: row.field(actor())?,
            company: row.optional_field(company())?,
            provenance: row.field(provenance())?,
        })
    }
}
impl WorthQueryApplicationProjection<Schema, ScopeQuery> for ScopeValue {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, ScopeQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self(row.field(scope_actor())?))
    }
}
fn graph_identity(input: ReadGraph) -> String {
    input.0
}
fn scope_identity(input: ReadScope) -> String {
    input.0
}
worth_query_query_binding!(pub GraphRead for ReadGraph,schema Schema,identity "worth.query.signed-path.graph-read-binding.v1",input GraphInput,query GraphQuery,parameters Parameters=>|_|ApplicationQueryParameterSet::new(),result GraphResult,principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,scope Row,RowFacts,RowIdentity,String,ReadOnly,NoApplicationUnit,field RowIdentity::reference(),value graph_identity,limits results 32);
worth_query_query_binding!(pub ScopeRead for ReadScope,schema Schema,identity "worth.query.signed-path.scope-read-binding.v1",input ScopeInput,query ScopeQuery,parameters Parameters=>|_|ApplicationQueryParameterSet::new(),result ScopeResult,principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,scope Row,RowFacts,RowIdentity,String,ReadOnly,NoApplicationUnit,field RowIdentity::reference(),value scope_identity,limits results 32);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let graph = ApplicationQueryDefinitionBuilder::declare(GraphQuery::reference())
        .root(GraphEvidence::reference())
        .scope(Row::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                Schema,
                GraphQuery,
                GraphEvidence,
                GraphValue,
                GraphResult,
            >::new(GraphEvidence::reference())
            .field(value())
            .field(revision())
            .field(actor())
            .optional_field(company())
            .field(provenance())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(Row::reference()).forward(GraphHistory::reference()),
        )
        .build()
        .unwrap();
    let scope = ApplicationQueryDefinitionBuilder::declare(ScopeQuery::reference())
        .root(ScopeEvidence::reference())
        .scope(Row::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                Schema,
                ScopeQuery,
                ScopeEvidence,
                ScopeValue,
                ScopeResult,
            >::new(ScopeEvidence::reference())
            .field(scope_actor())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(Row::reference()).forward(ScopeHistory::reference()),
        )
        .build()
        .unwrap();
    schema
        .application_query(graph)
        .application_query(scope)
        .application_query_binding::<GraphRead>()
        .application_query_binding::<ScopeRead>()
}
