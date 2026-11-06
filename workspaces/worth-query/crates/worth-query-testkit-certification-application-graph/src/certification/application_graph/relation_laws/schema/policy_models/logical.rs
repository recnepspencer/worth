//! One relation law model.
use super::super::*;
pub struct LogicalRecords;
impl Fragment for LogicalRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (ExternalMapping, Principal, Leaf, LogicalNode);
}
pub struct LogicalNode;
impl Entity for LogicalNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (LogicalPrecedes, LogicalFeeds);
}
pub struct LogicalPrecedes;
impl Relation for LogicalPrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = LogicalNode;
    const SELF_EDGES: bool = <super::super::super::logical::Logical as LawPolicy>::INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::super::logical::Logical as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::super::logical::Logical as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::super::logical::Logical as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::super::logical::Logical as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool =
        <super::super::super::logical::Logical as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::super::logical::Logical as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::super::logical::Logical as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::super::logical::Logical as LawPolicy>::INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::super::logical::Logical as LawPolicy>::INTEGRITY.reaches_at_least;
}
pub struct LogicalFeeds;
impl Relation for LogicalFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool = <super::super::super::logical::Logical as LawPolicy>::FEEDS_INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::super::logical::Logical as LawPolicy>::FEEDS_INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::super::logical::Logical as LawPolicy>::FEEDS_INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::super::logical::Logical as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool =
        <super::super::super::logical::Logical as LawPolicy>::FEEDS_INTEGRITY.acyclic;
    const SYMMETRIC: bool =
        <super::super::super::logical::Logical as LawPolicy>::FEEDS_INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::super::logical::Logical as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::super::logical::Logical as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::super::logical::Logical as LawPolicy>::FEEDS_INTEGRITY
            .same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::super::logical::Logical as LawPolicy>::FEEDS_INTEGRITY.reaches_at_least;
}

impl GraphModel for LogicalRecords {
    type Node = LogicalNode;
    type Precedes = LogicalPrecedes;
    type Feeds = LogicalFeeds;
    type Key = FieldSelection<Self, LogicalNode, NodeFacts, u64>;
    type Weight = RelationFieldSelection<Self, LogicalNode, LogicalPrecedes, EdgeFacts, u64>;
    fn node_key() -> Self::Key {
        Self::Key::at("NodeKey")
    }
    fn edge_weight() -> Option<Self::Weight> {
        None
    }
}
impl OperationCreates<super::super::super::creation::CreateNode> for LogicalNode {}
impl OperationReads<super::super::super::feeding::EditFeeds> for LogicalNode {}
impl OperationDeletes<super::super::super::feeding::EditFeeds> for LogicalNode {}
impl OperationReads<super::super::super::operation::EditEdges> for LogicalPrecedes {}
impl OperationLinks<super::super::super::operation::EditEdges> for LogicalPrecedes {}
impl OperationUnlinks<super::super::super::operation::EditEdges> for LogicalPrecedes {}
impl OperationLinks<super::super::super::creation::CreateNode> for LogicalPrecedes {}
impl OperationReads<super::super::super::feeding::EditFeeds> for LogicalFeeds {}
impl OperationLinks<super::super::super::feeding::EditFeeds> for LogicalFeeds {}
impl OperationUnlinks<super::super::super::feeding::EditFeeds> for LogicalFeeds {}
