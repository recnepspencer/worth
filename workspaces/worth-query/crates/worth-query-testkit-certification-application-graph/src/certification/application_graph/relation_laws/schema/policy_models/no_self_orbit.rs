//! One relation law model.
use super::super::*;
pub struct NoSelfOrbitRecords;
impl Fragment for NoSelfOrbitRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (ExternalMapping, Principal, Leaf, NoSelfOrbitNode);
}
pub struct NoSelfOrbitNode;
impl Entity for NoSelfOrbitNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (NoSelfOrbitPrecedes, NoSelfOrbitFeeds);
}
pub struct NoSelfOrbitPrecedes;
impl Relation for NoSelfOrbitPrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = NoSelfOrbitNode;
    const SELF_EDGES: bool = <super::super::NoSelfOrbit as LawPolicy>::INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::NoSelfOrbit as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::NoSelfOrbit as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::NoSelfOrbit as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::NoSelfOrbit as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool = <super::super::NoSelfOrbit as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::NoSelfOrbit as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::NoSelfOrbit as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::NoSelfOrbit as LawPolicy>::INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::NoSelfOrbit as LawPolicy>::INTEGRITY.reaches_at_least;
}
pub struct NoSelfOrbitFeeds;
impl Relation for NoSelfOrbitFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool = <super::super::NoSelfOrbit as LawPolicy>::FEEDS_INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::NoSelfOrbit as LawPolicy>::FEEDS_INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::NoSelfOrbit as LawPolicy>::FEEDS_INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::NoSelfOrbit as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::NoSelfOrbit as LawPolicy>::FEEDS_INTEGRITY.acyclic;
    const SYMMETRIC: bool = <super::super::NoSelfOrbit as LawPolicy>::FEEDS_INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::NoSelfOrbit as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::NoSelfOrbit as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::NoSelfOrbit as LawPolicy>::FEEDS_INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::NoSelfOrbit as LawPolicy>::FEEDS_INTEGRITY.reaches_at_least;
}

impl GraphModel for NoSelfOrbitRecords {
    type Node = NoSelfOrbitNode;
    type Precedes = NoSelfOrbitPrecedes;
    type Feeds = NoSelfOrbitFeeds;
    type Key = FieldSelection<Self, NoSelfOrbitNode, NodeFacts, u64>;
    type Weight =
        RelationFieldSelection<Self, NoSelfOrbitNode, NoSelfOrbitPrecedes, EdgeFacts, u64>;
    fn node_key() -> Self::Key {
        Self::Key::at("NodeKey")
    }
    fn edge_weight() -> Option<Self::Weight> {
        None
    }
}
impl OperationCreates<super::super::super::creation::CreateNode> for NoSelfOrbitNode {}
impl OperationReads<super::super::super::feeding::EditFeeds> for NoSelfOrbitNode {}
impl OperationDeletes<super::super::super::feeding::EditFeeds> for NoSelfOrbitNode {}
impl OperationReads<super::super::super::operation::EditEdges> for NoSelfOrbitPrecedes {}
impl OperationLinks<super::super::super::operation::EditEdges> for NoSelfOrbitPrecedes {}
impl OperationUnlinks<super::super::super::operation::EditEdges> for NoSelfOrbitPrecedes {}
impl OperationLinks<super::super::super::creation::CreateNode> for NoSelfOrbitPrecedes {}
impl OperationReads<super::super::super::feeding::EditFeeds> for NoSelfOrbitFeeds {}
impl OperationLinks<super::super::super::feeding::EditFeeds> for NoSelfOrbitFeeds {}
impl OperationUnlinks<super::super::super::feeding::EditFeeds> for NoSelfOrbitFeeds {}
