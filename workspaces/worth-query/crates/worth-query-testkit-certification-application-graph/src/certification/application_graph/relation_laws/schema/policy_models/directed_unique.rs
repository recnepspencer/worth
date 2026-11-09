//! One relation law model.
use super::super::*;
pub struct DirectedUniqueRecords;
impl Fragment for DirectedUniqueRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (ExternalMapping, Principal, Leaf, DirectedUniqueNode);
}
pub struct DirectedUniqueNode;
impl Entity for DirectedUniqueNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (DirectedUniquePrecedes, DirectedUniqueFeeds);
}
pub struct DirectedUniquePrecedes;
impl Relation for DirectedUniquePrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = DirectedUniqueNode;
    const SELF_EDGES: bool = <super::super::DirectedUnique as LawPolicy>::INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::DirectedUnique as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::DirectedUnique as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::DirectedUnique as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::DirectedUnique as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool = <super::super::DirectedUnique as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::DirectedUnique as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::DirectedUnique as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::DirectedUnique as LawPolicy>::INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::DirectedUnique as LawPolicy>::INTEGRITY.reaches_at_least;
}
pub struct DirectedUniqueFeeds;
impl Relation for DirectedUniqueFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool = <super::super::DirectedUnique as LawPolicy>::FEEDS_INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::DirectedUnique as LawPolicy>::FEEDS_INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::DirectedUnique as LawPolicy>::FEEDS_INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::DirectedUnique as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::DirectedUnique as LawPolicy>::FEEDS_INTEGRITY.acyclic;
    const SYMMETRIC: bool = <super::super::DirectedUnique as LawPolicy>::FEEDS_INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::DirectedUnique as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::DirectedUnique as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::DirectedUnique as LawPolicy>::FEEDS_INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::DirectedUnique as LawPolicy>::FEEDS_INTEGRITY.reaches_at_least;
}

impl GraphModel for DirectedUniqueRecords {
    type Node = DirectedUniqueNode;
    type Precedes = DirectedUniquePrecedes;
    type Feeds = DirectedUniqueFeeds;
    type Key = FieldSelection<Self, DirectedUniqueNode, NodeFacts, u64>;
    type Weight =
        RelationFieldSelection<Self, DirectedUniqueNode, DirectedUniquePrecedes, EdgeFacts, u64>;
    fn node_key() -> Self::Key {
        Self::Key::at("NodeKey")
    }
    fn edge_weight() -> Option<Self::Weight> {
        None
    }
}
impl OperationCreates<super::super::super::creation::CreateNode> for DirectedUniqueNode {}
impl OperationReads<super::super::super::feeding::EditFeeds> for DirectedUniqueNode {}
impl OperationDeletes<super::super::super::feeding::EditFeeds> for DirectedUniqueNode {}
impl OperationReads<super::super::super::operation::EditEdges> for DirectedUniquePrecedes {}
impl OperationLinks<super::super::super::operation::EditEdges> for DirectedUniquePrecedes {}
impl OperationUnlinks<super::super::super::operation::EditEdges> for DirectedUniquePrecedes {}
impl OperationLinks<super::super::super::creation::CreateNode> for DirectedUniquePrecedes {}
impl OperationReads<super::super::super::feeding::EditFeeds> for DirectedUniqueFeeds {}
impl OperationLinks<super::super::super::feeding::EditFeeds> for DirectedUniqueFeeds {}
impl OperationUnlinks<super::super::super::feeding::EditFeeds> for DirectedUniqueFeeds {}
