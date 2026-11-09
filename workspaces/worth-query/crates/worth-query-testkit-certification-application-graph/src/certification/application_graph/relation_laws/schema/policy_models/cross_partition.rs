//! One relation law model.
use super::super::*;
pub struct CrossPartitionRecords;
impl Fragment for CrossPartitionRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (ExternalMapping, Principal, Leaf, CrossPartitionNode);
}
pub struct CrossPartitionNode;
impl Entity for CrossPartitionNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (CrossPartitionPrecedes, CrossPartitionFeeds);
}
pub struct CrossPartitionPrecedes;
impl Relation for CrossPartitionPrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = CrossPartitionNode;
    const SELF_EDGES: bool = <super::super::partition::CrossPartition as LawPolicy>::INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::partition::CrossPartition as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::partition::CrossPartition as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::partition::CrossPartition as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::partition::CrossPartition as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool =
        <super::super::partition::CrossPartition as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::partition::CrossPartition as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::partition::CrossPartition as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::partition::CrossPartition as LawPolicy>::INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::partition::CrossPartition as LawPolicy>::INTEGRITY.reaches_at_least;
}
pub struct CrossPartitionFeeds;
impl Relation for CrossPartitionFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool =
        <super::super::partition::CrossPartition as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::partition::CrossPartition as LawPolicy>::FEEDS_INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::partition::CrossPartition as LawPolicy>::FEEDS_INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::partition::CrossPartition as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool =
        <super::super::partition::CrossPartition as LawPolicy>::FEEDS_INTEGRITY.acyclic;
    const SYMMETRIC: bool =
        <super::super::partition::CrossPartition as LawPolicy>::FEEDS_INTEGRITY.symmetric;
    const ORBIT: bool =
        <super::super::partition::CrossPartition as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::partition::CrossPartition as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::partition::CrossPartition as LawPolicy>::FEEDS_INTEGRITY
            .same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::partition::CrossPartition as LawPolicy>::FEEDS_INTEGRITY.reaches_at_least;
}

impl GraphModel for CrossPartitionRecords {
    type Node = CrossPartitionNode;
    type Precedes = CrossPartitionPrecedes;
    type Feeds = CrossPartitionFeeds;
    type Key = FieldSelection<Self, CrossPartitionNode, NodeFacts, u64>;
    type Weight =
        RelationFieldSelection<Self, CrossPartitionNode, CrossPartitionPrecedes, EdgeFacts, u64>;
    fn node_key() -> Self::Key {
        Self::Key::at("NodeKey")
    }
    fn edge_weight() -> Option<Self::Weight> {
        None
    }
}
impl OperationCreates<super::super::super::creation::CreateNode> for CrossPartitionNode {}
impl OperationReads<super::super::super::feeding::EditFeeds> for CrossPartitionNode {}
impl OperationDeletes<super::super::super::feeding::EditFeeds> for CrossPartitionNode {}
impl OperationReads<super::super::super::operation::EditEdges> for CrossPartitionPrecedes {}
impl OperationLinks<super::super::super::operation::EditEdges> for CrossPartitionPrecedes {}
impl OperationUnlinks<super::super::super::operation::EditEdges> for CrossPartitionPrecedes {}
impl OperationLinks<super::super::super::creation::CreateNode> for CrossPartitionPrecedes {}
impl OperationReads<super::super::super::feeding::EditFeeds> for CrossPartitionFeeds {}
impl OperationLinks<super::super::super::feeding::EditFeeds> for CrossPartitionFeeds {}
impl OperationUnlinks<super::super::super::feeding::EditFeeds> for CrossPartitionFeeds {}
