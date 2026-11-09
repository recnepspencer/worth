//! Each law fixture carries its actual typed node and outgoing relation roster.
use super::super::{creation::CreateNode, feeding::EditFeeds, operation::EditEdges};
use super::*;
pub trait GraphModel: Fragment {
    type Node: Entity
        + OperationCreates<CreateNode>
        + OperationReads<EditFeeds>
        + OperationDeletes<EditFeeds>;
    type Precedes: Relation<To = Self::Node>
        + OperationReads<EditEdges>
        + OperationLinks<EditEdges>
        + OperationUnlinks<EditEdges>
        + OperationLinks<CreateNode>;
    type Feeds: Relation<To = Leaf>
        + OperationReads<EditFeeds>
        + OperationLinks<EditFeeds>
        + OperationUnlinks<EditFeeds>;
    type Key: RecordSelection<
        Entity = Self::Node,
        Aspect = NodeFacts,
        Value = u64,
        Binding = U64ApplicationValueBinding,
    >;
    type Weight: RelationSelection<
        Entity = Self::Precedes,
        From = Self::Node,
        To = Self::Node,
        Value = u64,
        Binding = U64ApplicationValueBinding,
    >;
    fn node_key() -> Self::Key;
    fn edge_weight() -> Option<Self::Weight>;
}
pub type GraphNode<S> = <<S as GraphSchemaBinding>::Model as GraphModel>::Node;
pub type GraphPrecedes<S> = <<S as GraphSchemaBinding>::Model as GraphModel>::Precedes;
pub type GraphFeeds<S> = <<S as GraphSchemaBinding>::Model as GraphModel>::Feeds;
pub type GraphKey<S> = <<S as GraphSchemaBinding>::Model as GraphModel>::Key;
pub type GraphWeight<S> = <<S as GraphSchemaBinding>::Model as GraphModel>::Weight;
pub fn node_key<S: GraphSchemaBinding>() -> GraphKey<S> {
    S::Model::node_key()
}
pub fn edge_weight<S: GraphSchemaBinding>() -> Option<GraphWeight<S>> {
    S::Model::edge_weight()
}
pub fn node<S: GraphSchemaBinding>() -> ApplicationEntityRef<S, GraphNode<S>> {
    ApplicationEntityRef::from_schema_identifier(<GraphNode<S> as Entity>::NAME)
}
pub fn precedes<S: GraphSchemaBinding>(
) -> ApplicationRelationRef<S, GraphPrecedes<S>, GraphNode<S>, GraphNode<S>> {
    ApplicationRelationRef::from_schema_identifiers(
        <GraphPrecedes<S> as Relation>::NAME,
        <GraphNode<S> as Entity>::NAME,
        <GraphNode<S> as Entity>::NAME,
        fragment::relation_integrity::<GraphPrecedes<S>>(),
    )
}
pub fn feeds<S: GraphSchemaBinding>() -> ApplicationRelationRef<S, GraphFeeds<S>, GraphNode<S>, Leaf>
{
    ApplicationRelationRef::from_schema_identifiers(
        <GraphFeeds<S> as Relation>::NAME,
        <GraphNode<S> as Entity>::NAME,
        <Leaf as Entity>::NAME,
        fragment::relation_integrity::<GraphFeeds<S>>(),
    )
}
