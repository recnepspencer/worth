use super::*;
#[derive(Clone, Debug, PartialEq, Eq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct MemberRow {
    pub key: String,
    pub value: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct GraphRow {
    pub key: String,
    pub value: u64,
    pub members: Vec<MemberRow>,
}
worth_query_structured_value_binding!(pub RowBinding for GraphRow { identity: "source-subgraph.graph-row.v1" });
macro_rules! graph_query {
    ($module:ident, $entity:ident, $facts:ident, $key:ident, $value:ident, $relation:ident, $semantic:literal, $query_id:literal, $binding_id:literal) => {
        pub mod $module {
            use super::*;
            use super::super::schema::{$entity as Entity, $facts as Facts, $key as Key, $value as Value, $relation as Relation};
            worth_query_application_query!(pub Query for Schema, identity $query_id, parameters ParametersBinding, result RowBinding, scope Entity => $semantic, name $semantic);
            const BINDING_ID: &str = $binding_id;
            include!("graph_query.rs");
        }
    };
}
graph_query!(
    source,
    Source,
    SourceFacts,
    SourceKey,
    SourceValue,
    SourceMember,
    "Source",
    "source-subgraph.source-read.v1",
    "source-subgraph.source-read-binding.v1"
);
graph_query!(
    generated,
    Generated,
    GeneratedFacts,
    GeneratedKey,
    GeneratedValue,
    GeneratedMember,
    "Generated",
    "source-subgraph.generated-read.v1",
    "source-subgraph.generated-read-binding.v1"
);
