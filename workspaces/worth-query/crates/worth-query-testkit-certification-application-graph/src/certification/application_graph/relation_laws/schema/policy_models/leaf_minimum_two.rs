//! One relation law model.
use super::super::*;
pub struct LeafMinimumTwoRecords;
impl Fragment for LeafMinimumTwoRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (ExternalMapping, Principal, Leaf, LeafMinimumTwoNode);
}
pub struct LeafMinimumTwoNode;
impl Entity for LeafMinimumTwoNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (LeafMinimumTwoPrecedes, LeafMinimumTwoFeeds);
}
pub struct LeafMinimumTwoPrecedes;
impl Relation for LeafMinimumTwoPrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = LeafMinimumTwoNode;
    const SELF_EDGES: bool = <super::super::minimum::LeafMinimumTwo as LawPolicy>::INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::minimum::LeafMinimumTwo as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::minimum::LeafMinimumTwo as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::minimum::LeafMinimumTwo as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::minimum::LeafMinimumTwo as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool =
        <super::super::minimum::LeafMinimumTwo as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::minimum::LeafMinimumTwo as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::minimum::LeafMinimumTwo as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::minimum::LeafMinimumTwo as LawPolicy>::INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::minimum::LeafMinimumTwo as LawPolicy>::INTEGRITY.reaches_at_least;
}
pub struct LeafMinimumTwoFeeds;
impl Relation for LeafMinimumTwoFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool = <super::super::minimum::LeafMinimumTwo as LawPolicy>::FEEDS_INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::minimum::LeafMinimumTwo as LawPolicy>::FEEDS_INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::minimum::LeafMinimumTwo as LawPolicy>::FEEDS_INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::minimum::LeafMinimumTwo as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool =
        <super::super::minimum::LeafMinimumTwo as LawPolicy>::FEEDS_INTEGRITY.acyclic;
    const SYMMETRIC: bool =
        <super::super::minimum::LeafMinimumTwo as LawPolicy>::FEEDS_INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::minimum::LeafMinimumTwo as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::minimum::LeafMinimumTwo as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::minimum::LeafMinimumTwo as LawPolicy>::FEEDS_INTEGRITY
            .same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::minimum::LeafMinimumTwo as LawPolicy>::FEEDS_INTEGRITY.reaches_at_least;
}

impl GraphModel for LeafMinimumTwoRecords {
    type Node = LeafMinimumTwoNode;
    type Precedes = LeafMinimumTwoPrecedes;
    type Feeds = LeafMinimumTwoFeeds;
    type Key = FieldSelection<Self, LeafMinimumTwoNode, NodeFacts, u64>;
    type Weight =
        RelationFieldSelection<Self, LeafMinimumTwoNode, LeafMinimumTwoPrecedes, EdgeFacts, u64>;
    fn node_key() -> Self::Key {
        Self::Key::at("NodeKey")
    }
    fn edge_weight() -> Option<Self::Weight> {
        None
    }
}
impl OperationCreates<super::super::super::creation::CreateNode> for LeafMinimumTwoNode {}
impl OperationReads<super::super::super::feeding::EditFeeds> for LeafMinimumTwoNode {}
impl OperationDeletes<super::super::super::feeding::EditFeeds> for LeafMinimumTwoNode {}
impl OperationReads<super::super::super::operation::EditEdges> for LeafMinimumTwoPrecedes {}
impl OperationLinks<super::super::super::operation::EditEdges> for LeafMinimumTwoPrecedes {}
impl OperationUnlinks<super::super::super::operation::EditEdges> for LeafMinimumTwoPrecedes {}
impl OperationLinks<super::super::super::creation::CreateNode> for LeafMinimumTwoPrecedes {}
impl OperationReads<super::super::super::feeding::EditFeeds> for LeafMinimumTwoFeeds {}
impl OperationLinks<super::super::super::feeding::EditFeeds> for LeafMinimumTwoFeeds {}
impl OperationUnlinks<super::super::super::feeding::EditFeeds> for LeafMinimumTwoFeeds {}
