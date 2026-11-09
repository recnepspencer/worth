//! One relation law model.
use super::super::*;
pub struct IsolatedRecords;
impl Fragment for IsolatedRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (ExternalMapping, Principal, Leaf, IsolatedNode);
}
pub struct IsolatedNode;
impl Entity for IsolatedNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (IsolatedPrecedes, IsolatedFeeds);
}
pub struct IsolatedPrecedes;
impl Relation for IsolatedPrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = IsolatedNode;
    const SELF_EDGES: bool = <super::super::partition::Isolated as LawPolicy>::INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::partition::Isolated as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::partition::Isolated as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::partition::Isolated as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::partition::Isolated as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool = <super::super::partition::Isolated as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::partition::Isolated as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::partition::Isolated as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::partition::Isolated as LawPolicy>::INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::partition::Isolated as LawPolicy>::INTEGRITY.reaches_at_least;
}
pub struct IsolatedFeeds;
impl Relation for IsolatedFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool = <super::super::partition::Isolated as LawPolicy>::FEEDS_INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::partition::Isolated as LawPolicy>::FEEDS_INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::partition::Isolated as LawPolicy>::FEEDS_INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::partition::Isolated as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::partition::Isolated as LawPolicy>::FEEDS_INTEGRITY.acyclic;
    const SYMMETRIC: bool =
        <super::super::partition::Isolated as LawPolicy>::FEEDS_INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::partition::Isolated as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::partition::Isolated as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::partition::Isolated as LawPolicy>::FEEDS_INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::partition::Isolated as LawPolicy>::FEEDS_INTEGRITY.reaches_at_least;
}

impl GraphModel for IsolatedRecords {
    type Node = IsolatedNode;
    type Precedes = IsolatedPrecedes;
    type Feeds = IsolatedFeeds;
    type Key = FieldSelection<Self, IsolatedNode, NodeFacts, u64>;
    type Weight = RelationFieldSelection<Self, IsolatedNode, IsolatedPrecedes, EdgeFacts, u64>;
    fn node_key() -> Self::Key {
        Self::Key::at("NodeKey")
    }
    fn edge_weight() -> Option<Self::Weight> {
        None
    }
}
impl OperationCreates<super::super::super::creation::CreateNode> for IsolatedNode {}
impl OperationReads<super::super::super::feeding::EditFeeds> for IsolatedNode {}
impl OperationDeletes<super::super::super::feeding::EditFeeds> for IsolatedNode {}
impl OperationReads<super::super::super::operation::EditEdges> for IsolatedPrecedes {}
impl OperationLinks<super::super::super::operation::EditEdges> for IsolatedPrecedes {}
impl OperationUnlinks<super::super::super::operation::EditEdges> for IsolatedPrecedes {}
impl OperationLinks<super::super::super::creation::CreateNode> for IsolatedPrecedes {}
impl OperationReads<super::super::super::feeding::EditFeeds> for IsolatedFeeds {}
impl OperationLinks<super::super::super::feeding::EditFeeds> for IsolatedFeeds {}
impl OperationUnlinks<super::super::super::feeding::EditFeeds> for IsolatedFeeds {}
