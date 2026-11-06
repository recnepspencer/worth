//! One relation law model.
use super::super::*;
pub struct MinimumTwoRecords;
impl Fragment for MinimumTwoRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (ExternalMapping, Principal, Leaf, MinimumTwoNode);
}
pub struct MinimumTwoNode;
impl Entity for MinimumTwoNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (MinimumTwoPrecedes, MinimumTwoFeeds);
}
pub struct MinimumTwoPrecedes;
impl Relation for MinimumTwoPrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = MinimumTwoNode;
    const SELF_EDGES: bool = <super::super::minimum::MinimumTwo as LawPolicy>::INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::minimum::MinimumTwo as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::minimum::MinimumTwo as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::minimum::MinimumTwo as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::minimum::MinimumTwo as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool = <super::super::minimum::MinimumTwo as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::minimum::MinimumTwo as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::minimum::MinimumTwo as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::minimum::MinimumTwo as LawPolicy>::INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::minimum::MinimumTwo as LawPolicy>::INTEGRITY.reaches_at_least;
}
pub struct MinimumTwoFeeds;
impl Relation for MinimumTwoFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool = <super::super::minimum::MinimumTwo as LawPolicy>::FEEDS_INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::minimum::MinimumTwo as LawPolicy>::FEEDS_INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::minimum::MinimumTwo as LawPolicy>::FEEDS_INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::minimum::MinimumTwo as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::minimum::MinimumTwo as LawPolicy>::FEEDS_INTEGRITY.acyclic;
    const SYMMETRIC: bool =
        <super::super::minimum::MinimumTwo as LawPolicy>::FEEDS_INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::minimum::MinimumTwo as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::minimum::MinimumTwo as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::minimum::MinimumTwo as LawPolicy>::FEEDS_INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::minimum::MinimumTwo as LawPolicy>::FEEDS_INTEGRITY.reaches_at_least;
}

impl GraphModel for MinimumTwoRecords {
    type Node = MinimumTwoNode;
    type Precedes = MinimumTwoPrecedes;
    type Feeds = MinimumTwoFeeds;
    type Key = FieldSelection<Self, MinimumTwoNode, NodeFacts, u64>;
    type Weight = RelationFieldSelection<Self, MinimumTwoNode, MinimumTwoPrecedes, EdgeFacts, u64>;
    fn node_key() -> Self::Key {
        Self::Key::at("NodeKey")
    }
    fn edge_weight() -> Option<Self::Weight> {
        None
    }
}
impl OperationCreates<super::super::super::creation::CreateNode> for MinimumTwoNode {}
impl OperationReads<super::super::super::feeding::EditFeeds> for MinimumTwoNode {}
impl OperationDeletes<super::super::super::feeding::EditFeeds> for MinimumTwoNode {}
impl OperationReads<super::super::super::operation::EditEdges> for MinimumTwoPrecedes {}
impl OperationLinks<super::super::super::operation::EditEdges> for MinimumTwoPrecedes {}
impl OperationUnlinks<super::super::super::operation::EditEdges> for MinimumTwoPrecedes {}
impl OperationLinks<super::super::super::creation::CreateNode> for MinimumTwoPrecedes {}
impl OperationReads<super::super::super::feeding::EditFeeds> for MinimumTwoFeeds {}
impl OperationLinks<super::super::super::feeding::EditFeeds> for MinimumTwoFeeds {}
impl OperationUnlinks<super::super::super::feeding::EditFeeds> for MinimumTwoFeeds {}
