//! One relation law model.
use super::super::*;
pub struct AcyclicRecords;
impl Fragment for AcyclicRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (ExternalMapping, Principal, Leaf, AcyclicNode);
}
pub struct AcyclicNode;
impl Entity for AcyclicNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (AcyclicPrecedes, AcyclicFeeds);
}
pub struct AcyclicPrecedes;
impl Relation for AcyclicPrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = AcyclicNode;
    const SELF_EDGES: bool = <super::super::Acyclic as LawPolicy>::INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::Acyclic as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::Acyclic as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::Acyclic as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::Acyclic as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool = <super::super::Acyclic as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::Acyclic as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::Acyclic as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::Acyclic as LawPolicy>::INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::Acyclic as LawPolicy>::INTEGRITY.reaches_at_least;
}
pub struct AcyclicFeeds;
impl Relation for AcyclicFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool = <super::super::Acyclic as LawPolicy>::FEEDS_INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::Acyclic as LawPolicy>::FEEDS_INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::Acyclic as LawPolicy>::FEEDS_INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::Acyclic as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::Acyclic as LawPolicy>::FEEDS_INTEGRITY.acyclic;
    const SYMMETRIC: bool = <super::super::Acyclic as LawPolicy>::FEEDS_INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::Acyclic as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::Acyclic as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::Acyclic as LawPolicy>::FEEDS_INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::Acyclic as LawPolicy>::FEEDS_INTEGRITY.reaches_at_least;
}

impl GraphModel for AcyclicRecords {
    type Node = AcyclicNode;
    type Precedes = AcyclicPrecedes;
    type Feeds = AcyclicFeeds;
    type Key = FieldSelection<Self, AcyclicNode, NodeFacts, u64>;
    type Weight = RelationFieldSelection<Self, AcyclicNode, AcyclicPrecedes, EdgeFacts, u64>;
    fn node_key() -> Self::Key {
        Self::Key::at("NodeKey")
    }
    fn edge_weight() -> Option<Self::Weight> {
        None
    }
}
impl OperationCreates<super::super::super::creation::CreateNode> for AcyclicNode {}
impl OperationReads<super::super::super::feeding::EditFeeds> for AcyclicNode {}
impl OperationDeletes<super::super::super::feeding::EditFeeds> for AcyclicNode {}
impl OperationReads<super::super::super::operation::EditEdges> for AcyclicPrecedes {}
impl OperationLinks<super::super::super::operation::EditEdges> for AcyclicPrecedes {}
impl OperationUnlinks<super::super::super::operation::EditEdges> for AcyclicPrecedes {}
impl OperationLinks<super::super::super::creation::CreateNode> for AcyclicPrecedes {}
impl OperationReads<super::super::super::feeding::EditFeeds> for AcyclicFeeds {}
impl OperationLinks<super::super::super::feeding::EditFeeds> for AcyclicFeeds {}
impl OperationUnlinks<super::super::super::feeding::EditFeeds> for AcyclicFeeds {}
