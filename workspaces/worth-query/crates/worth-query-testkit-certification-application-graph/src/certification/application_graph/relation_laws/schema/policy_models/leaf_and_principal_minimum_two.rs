//! One relation law model.
use super::super::*;
pub struct LeafAndPrincipalMinimumTwoRecords;
impl Fragment for LeafAndPrincipalMinimumTwoRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (
        ExternalMapping,
        Principal,
        Leaf,
        LeafAndPrincipalMinimumTwoNode,
    );
}
pub struct LeafAndPrincipalMinimumTwoNode;
impl Entity for LeafAndPrincipalMinimumTwoNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (
        LeafAndPrincipalMinimumTwoPrecedes,
        LeafAndPrincipalMinimumTwoFeeds,
    );
}
pub struct LeafAndPrincipalMinimumTwoPrecedes;
impl Relation for LeafAndPrincipalMinimumTwoPrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = LeafAndPrincipalMinimumTwoNode;
    const SELF_EDGES: bool =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::INTEGRITY
            .endpoints
            .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::INTEGRITY
            .same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::INTEGRITY
            .reaches_at_least;
}
pub struct LeafAndPrincipalMinimumTwoFeeds;
impl Relation for LeafAndPrincipalMinimumTwoFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::FEEDS_INTEGRITY
            .cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::FEEDS_INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::FEEDS_INTEGRITY.acyclic;
    const SYMMETRIC: bool =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::FEEDS_INTEGRITY.symmetric;
    const ORBIT: bool =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::FEEDS_INTEGRITY
            .same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::minimum::LeafAndPrincipalMinimumTwo as LawPolicy>::FEEDS_INTEGRITY
            .reaches_at_least;
}

impl GraphModel for LeafAndPrincipalMinimumTwoRecords {
    type Node = LeafAndPrincipalMinimumTwoNode;
    type Precedes = LeafAndPrincipalMinimumTwoPrecedes;
    type Feeds = LeafAndPrincipalMinimumTwoFeeds;
    type Key = FieldSelection<Self, LeafAndPrincipalMinimumTwoNode, NodeFacts, u64>;
    type Weight = RelationFieldSelection<
        Self,
        LeafAndPrincipalMinimumTwoNode,
        LeafAndPrincipalMinimumTwoPrecedes,
        EdgeFacts,
        u64,
    >;
    fn node_key() -> Self::Key {
        Self::Key::at("NodeKey")
    }
    fn edge_weight() -> Option<Self::Weight> {
        None
    }
}
impl OperationCreates<super::super::super::creation::CreateNode>
    for LeafAndPrincipalMinimumTwoNode
{
}
impl OperationReads<super::super::super::feeding::EditFeeds> for LeafAndPrincipalMinimumTwoNode {}
impl OperationDeletes<super::super::super::feeding::EditFeeds> for LeafAndPrincipalMinimumTwoNode {}
impl OperationReads<super::super::super::operation::EditEdges>
    for LeafAndPrincipalMinimumTwoPrecedes
{
}
impl OperationLinks<super::super::super::operation::EditEdges>
    for LeafAndPrincipalMinimumTwoPrecedes
{
}
impl OperationUnlinks<super::super::super::operation::EditEdges>
    for LeafAndPrincipalMinimumTwoPrecedes
{
}
impl OperationLinks<super::super::super::creation::CreateNode>
    for LeafAndPrincipalMinimumTwoPrecedes
{
}
impl OperationReads<super::super::super::feeding::EditFeeds> for LeafAndPrincipalMinimumTwoFeeds {}
impl OperationLinks<super::super::super::feeding::EditFeeds> for LeafAndPrincipalMinimumTwoFeeds {}
impl OperationUnlinks<super::super::super::feeding::EditFeeds> for LeafAndPrincipalMinimumTwoFeeds {}
