//! One relation law model.
use super::super::*;
pub struct OrbitRecords;
impl Fragment for OrbitRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (ExternalMapping, Principal, Leaf, OrbitNode);
}
pub struct OrbitNode;
impl Entity for OrbitNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (OrbitPrecedes, OrbitFeeds);
}
pub struct OrbitPrecedes;
impl Relation for OrbitPrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = OrbitNode;
    const SELF_EDGES: bool = <super::super::Orbit as LawPolicy>::INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::Orbit as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::Orbit as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::Orbit as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::Orbit as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool = <super::super::Orbit as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::Orbit as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::Orbit as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::Orbit as LawPolicy>::INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::Orbit as LawPolicy>::INTEGRITY.reaches_at_least;
}
pub struct OrbitFeeds;
impl Relation for OrbitFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool = <super::super::Orbit as LawPolicy>::FEEDS_INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::Orbit as LawPolicy>::FEEDS_INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::Orbit as LawPolicy>::FEEDS_INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::Orbit as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::Orbit as LawPolicy>::FEEDS_INTEGRITY.acyclic;
    const SYMMETRIC: bool = <super::super::Orbit as LawPolicy>::FEEDS_INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::Orbit as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::Orbit as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::Orbit as LawPolicy>::FEEDS_INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::Orbit as LawPolicy>::FEEDS_INTEGRITY.reaches_at_least;
}

impl GraphModel for OrbitRecords {
    type Node = OrbitNode;
    type Precedes = OrbitPrecedes;
    type Feeds = OrbitFeeds;
    type Key = FieldSelection<Self, OrbitNode, NodeFacts, u64>;
    type Weight = RelationFieldSelection<Self, OrbitNode, OrbitPrecedes, EdgeFacts, u64>;
    fn node_key() -> Self::Key {
        Self::Key::at("NodeKey")
    }
    fn edge_weight() -> Option<Self::Weight> {
        None
    }
}
impl OperationCreates<super::super::super::creation::CreateNode> for OrbitNode {}
impl OperationReads<super::super::super::feeding::EditFeeds> for OrbitNode {}
impl OperationDeletes<super::super::super::feeding::EditFeeds> for OrbitNode {}
impl OperationReads<super::super::super::operation::EditEdges> for OrbitPrecedes {}
impl OperationLinks<super::super::super::operation::EditEdges> for OrbitPrecedes {}
impl OperationUnlinks<super::super::super::operation::EditEdges> for OrbitPrecedes {}
impl OperationLinks<super::super::super::creation::CreateNode> for OrbitPrecedes {}
impl OperationReads<super::super::super::feeding::EditFeeds> for OrbitFeeds {}
impl OperationLinks<super::super::super::feeding::EditFeeds> for OrbitFeeds {}
impl OperationUnlinks<super::super::super::feeding::EditFeeds> for OrbitFeeds {}
