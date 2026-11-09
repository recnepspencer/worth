//! One relation law model.
use super::super::*;
pub struct DeletionProtectedRecords;
impl Fragment for DeletionProtectedRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (ExternalMapping, Principal, Leaf, DeletionProtectedNode);
}
pub struct DeletionProtectedNode;
impl Entity for DeletionProtectedNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (DeletionProtectedPrecedes, DeletionProtectedFeeds);
}
pub struct DeletionProtectedPrecedes;
impl Relation for DeletionProtectedPrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = DeletionProtectedNode;
    const SELF_EDGES: bool = <super::super::DeletionProtected as LawPolicy>::INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::DeletionProtected as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::DeletionProtected as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::DeletionProtected as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::DeletionProtected as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool = <super::super::DeletionProtected as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::DeletionProtected as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::DeletionProtected as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::DeletionProtected as LawPolicy>::INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::DeletionProtected as LawPolicy>::INTEGRITY.reaches_at_least;
}
pub struct DeletionProtectedFeeds;
impl Relation for DeletionProtectedFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool = <super::super::DeletionProtected as LawPolicy>::FEEDS_INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::DeletionProtected as LawPolicy>::FEEDS_INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::DeletionProtected as LawPolicy>::FEEDS_INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::DeletionProtected as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::DeletionProtected as LawPolicy>::FEEDS_INTEGRITY.acyclic;
    const SYMMETRIC: bool =
        <super::super::DeletionProtected as LawPolicy>::FEEDS_INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::DeletionProtected as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::DeletionProtected as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::DeletionProtected as LawPolicy>::FEEDS_INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::DeletionProtected as LawPolicy>::FEEDS_INTEGRITY.reaches_at_least;
}

impl GraphModel for DeletionProtectedRecords {
    type Node = DeletionProtectedNode;
    type Precedes = DeletionProtectedPrecedes;
    type Feeds = DeletionProtectedFeeds;
    type Key = FieldSelection<Self, DeletionProtectedNode, NodeFacts, u64>;
    type Weight = RelationFieldSelection<
        Self,
        DeletionProtectedNode,
        DeletionProtectedPrecedes,
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
impl OperationCreates<super::super::super::creation::CreateNode> for DeletionProtectedNode {}
impl OperationReads<super::super::super::feeding::EditFeeds> for DeletionProtectedNode {}
impl OperationDeletes<super::super::super::feeding::EditFeeds> for DeletionProtectedNode {}
impl OperationReads<super::super::super::operation::EditEdges> for DeletionProtectedPrecedes {}
impl OperationLinks<super::super::super::operation::EditEdges> for DeletionProtectedPrecedes {}
impl OperationUnlinks<super::super::super::operation::EditEdges> for DeletionProtectedPrecedes {}
impl OperationLinks<super::super::super::creation::CreateNode> for DeletionProtectedPrecedes {}
impl OperationReads<super::super::super::feeding::EditFeeds> for DeletionProtectedFeeds {}
impl OperationLinks<super::super::super::feeding::EditFeeds> for DeletionProtectedFeeds {}
impl OperationUnlinks<super::super::super::feeding::EditFeeds> for DeletionProtectedFeeds {}
