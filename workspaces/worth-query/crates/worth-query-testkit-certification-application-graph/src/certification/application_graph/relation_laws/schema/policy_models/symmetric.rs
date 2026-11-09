//! One relation law model.
use super::super::*;
pub struct SymmetricRecords;
impl Fragment for SymmetricRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (ExternalMapping, Principal, Leaf, SymmetricNode);
}
pub struct SymmetricNode;
impl Entity for SymmetricNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (SymmetricPrecedes, SymmetricFeeds);
}
pub struct SymmetricPrecedes;
impl Relation for SymmetricPrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = SymmetricNode;
    const SELF_EDGES: bool = <super::super::Symmetric as LawPolicy>::INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::Symmetric as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::Symmetric as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::Symmetric as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::Symmetric as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool = <super::super::Symmetric as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::Symmetric as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::Symmetric as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::Symmetric as LawPolicy>::INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::Symmetric as LawPolicy>::INTEGRITY.reaches_at_least;
}
pub struct SymmetricFeeds;
impl Relation for SymmetricFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool = <super::super::Symmetric as LawPolicy>::FEEDS_INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::Symmetric as LawPolicy>::FEEDS_INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::Symmetric as LawPolicy>::FEEDS_INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::Symmetric as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::Symmetric as LawPolicy>::FEEDS_INTEGRITY.acyclic;
    const SYMMETRIC: bool = <super::super::Symmetric as LawPolicy>::FEEDS_INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::Symmetric as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::Symmetric as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::Symmetric as LawPolicy>::FEEDS_INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::Symmetric as LawPolicy>::FEEDS_INTEGRITY.reaches_at_least;
}

impl GraphModel for SymmetricRecords {
    type Node = SymmetricNode;
    type Precedes = SymmetricPrecedes;
    type Feeds = SymmetricFeeds;
    type Key = FieldSelection<Self, SymmetricNode, NodeFacts, u64>;
    type Weight = RelationFieldSelection<Self, SymmetricNode, SymmetricPrecedes, EdgeFacts, u64>;
    fn node_key() -> Self::Key {
        Self::Key::at("NodeKey")
    }
    fn edge_weight() -> Option<Self::Weight> {
        None
    }
}
impl OperationCreates<super::super::super::creation::CreateNode> for SymmetricNode {}
impl OperationReads<super::super::super::feeding::EditFeeds> for SymmetricNode {}
impl OperationDeletes<super::super::super::feeding::EditFeeds> for SymmetricNode {}
impl OperationReads<super::super::super::operation::EditEdges> for SymmetricPrecedes {}
impl OperationLinks<super::super::super::operation::EditEdges> for SymmetricPrecedes {}
impl OperationUnlinks<super::super::super::operation::EditEdges> for SymmetricPrecedes {}
impl OperationLinks<super::super::super::creation::CreateNode> for SymmetricPrecedes {}
impl OperationReads<super::super::super::feeding::EditFeeds> for SymmetricFeeds {}
impl OperationLinks<super::super::super::feeding::EditFeeds> for SymmetricFeeds {}
impl OperationUnlinks<super::super::super::feeding::EditFeeds> for SymmetricFeeds {}
