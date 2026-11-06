//! One record model owns field lowering and reusable scalar bindings.

use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct HistoryRecords;
impl fragment::Fragment for HistoryRecords {
    const NAME: &'static str = "history_records";
    type Bindings = HistoryRecordsBindings;
    type Members = (Node,);
}
pub struct HistoryRecordsBindings;
impl fragment::BindingSet for HistoryRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for HistoryRecordsBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
#[derive(Clone, Copy)]
pub struct Node;
impl fragment::Entity for Node {
    const NAME: &'static str = "Node";
    type Aspects = (NodeFacts,);
    type Relations = ();
}
impl Node {
    pub fn reference<Schema: super::SchemaBinding>() -> ApplicationEntityRef<Schema, Self> {
        entity(HistoryRecords, Self)
    }
}
pub struct NodeFacts;
impl fragment::Aspect for NodeFacts {
    const NAME: &'static str = "NodeFacts";
    type Row = NodeFactsValue;
}
impl NodeFacts {
    pub fn reference<Schema: super::SchemaBinding>() -> AspectSelection<HistoryRecords, Node, Self>
    {
        aspect(HistoryRecords, Node, Self)
    }
}
pub struct NodeFactsValue {
    pub node_key: u64,
    pub value: u64,
    pub applied: u64,
}
impl fragment::Record<NodeFactsValue> for HistoryRecordsBindings {
    fn record() -> impl fragment::Shape<NodeFactsValue, Self> {
        fragment::field("NodeKey", |row: &NodeFactsValue| row.node_key)
            .read_only()
            .and("Value", |row: &NodeFactsValue| row.value)
            .and("Applied", |row: &NodeFactsValue| row.applied)
            .build(
                |node_key, value, applied, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(NodeFactsValue {
                        node_key,
                        value,
                        applied,
                    })
                },
            )
    }
}
pub type NodeKey = FieldSelection<HistoryRecords, Node, NodeFacts, u64>;
pub type Value = FieldSelection<HistoryRecords, Node, NodeFacts, u64>;
pub type Applied = FieldSelection<HistoryRecords, Node, NodeFacts, u64>;
