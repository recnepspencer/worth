//! Current and historical evidence are read through separate native relations.
use super::{super::schema::*, signing::*};
use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_application_query,
    worth_query_portable_type, worth_query_query_binding, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Value(pub String, pub worth_relational::facade::identity::EntityId);
worth_query_structured_value_binding!(pub ResultBinding for Value {identity:"worth.query.reference.signature-result.v1"});
worth_query_structured_value_binding!(pub Parameters for () {identity:"worth.query.reference.signature-parameters.v1"});
macro_rules! query {
    ($input:ident,$binding:ident,$query:ident,$slot:ident,$field:ident,$definition:ident,$relation:ty,$input_binding:ident,$slot_id:literal,$query_id:literal,$binding_id:literal)=>{
        #[derive(Clone)]pub struct $input(pub String);
        impl $input {fn identity(input:Self)->String {input.0}}
        worth_query_structured_value_binding!(pub $input_binding for $input {identity:concat!("worth.query.reference.",stringify!($input))});
        pub struct $slot;worth_query_portable_type!($slot=>$slot_id);
        worth_query_application_query!(pub $query for Schema,identity $query_id,parameters Parameters,result ResultBinding,scope Row=>"Row",name $query_id);
        fn $field()->ApplicationQueryResultFieldRef<$query,$slot,Schema,Evidence,Facts,SignedIdentity,String,ReadOnly,EqualityPredicate,NoApplicationUnit> {ApplicationQueryResultFieldRef::new("identity",SignedIdentity::reference())}
        pub fn $definition()->ApplicationQueryDefinition<Schema,$query,(),Value,Row> {
            ApplicationQueryDefinitionBuilder::declare($query::reference()).root(Evidence::reference()).scope(Row::reference())
                .result_shape(ApplicationQueryResultShapeBuilder::<Schema,$query,Evidence,Value,ResultBinding>::new(Evidence::reference()).field($field()).build())
                .cardinality(ApplicationQueryCardinality::Many).dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1,1,128))
                .disclosure(ApplicationQueryDisclosureContract::public()).basis_support(ApplicationQueryBasisSupport::current_and_pinned()).lanes(ApplicationQueryLaneEligibility::one_shot()).public()
                .root_path(ApplicationQueryRootPath::from(Row::reference()).forward(<$relation>::reference())).build().unwrap()
        }
        impl WorthQueryApplicationProjection<Schema,$query> for Value {
            fn project(row:&WorthQueryApplicationProjectionRow<'_,Schema,$query>)->Result<Self,WorthQueryApplicationProjectionDenial> {Ok(Self(row.field($field())?,row.entity_id()))}
        }
        worth_query_query_binding!(pub $binding for $input,schema Schema,identity $binding_id,input $input_binding,query $query,parameters Parameters=>|_|ApplicationQueryParameterSet::new(),result ResultBinding,
            principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
            scope Row,RowFacts,RowIdentity,String,ReadOnly,NoApplicationUnit,field RowIdentity::reference(),value $input::identity,limits results 8);
    };
}
query!(
    ReadCurrent,
    CurrentRead,
    CurrentQuery,
    CurrentSlot,
    current,
    current_definition,
    Current,
    CurrentInput,
    "worth.query.reference.current-slot",
    "ReferenceCurrentQuery",
    "worth.query.reference.current-binding"
);
query!(
    ReadHistory,
    HistoryRead,
    HistoryQuery,
    HistorySlot,
    history,
    history_definition,
    History,
    HistoryInput,
    "worth.query.reference.history-slot",
    "ReferenceHistoryQuery",
    "worth.query.reference.history-binding"
);
pub fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    builder
        .application_query(current_definition())
        .application_query_binding::<CurrentRead>()
        .application_query(history_definition())
        .application_query_binding::<HistoryRead>()
}
