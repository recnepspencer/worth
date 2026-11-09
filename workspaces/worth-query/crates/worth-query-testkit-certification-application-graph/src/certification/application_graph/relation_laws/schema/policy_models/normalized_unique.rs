//! One relation law model.
use super::super::*;
pub struct NormalizedUniqueRecords;
impl Fragment for NormalizedUniqueRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (ExternalMapping, Principal, Leaf, NormalizedUniqueNode);
}
pub struct NormalizedUniqueNode;
impl Entity for NormalizedUniqueNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (NormalizedUniquePrecedes, NormalizedUniqueFeeds);
}
pub struct NormalizedUniquePrecedes;
impl Relation for NormalizedUniquePrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = NormalizedUniqueNode;
    const SELF_EDGES: bool = <super::super::NormalizedUnique as LawPolicy>::INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::NormalizedUnique as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::NormalizedUnique as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::NormalizedUnique as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::NormalizedUnique as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool = <super::super::NormalizedUnique as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::NormalizedUnique as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::NormalizedUnique as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::NormalizedUnique as LawPolicy>::INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::NormalizedUnique as LawPolicy>::INTEGRITY.reaches_at_least;
}
pub struct NormalizedUniqueFeeds;
impl Relation for NormalizedUniqueFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool = <super::super::NormalizedUnique as LawPolicy>::FEEDS_INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::NormalizedUnique as LawPolicy>::FEEDS_INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::NormalizedUnique as LawPolicy>::FEEDS_INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::NormalizedUnique as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::NormalizedUnique as LawPolicy>::FEEDS_INTEGRITY.acyclic;
    const SYMMETRIC: bool =
        <super::super::NormalizedUnique as LawPolicy>::FEEDS_INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::NormalizedUnique as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::NormalizedUnique as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::NormalizedUnique as LawPolicy>::FEEDS_INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::NormalizedUnique as LawPolicy>::FEEDS_INTEGRITY.reaches_at_least;
}

impl GraphModel for NormalizedUniqueRecords {
    type Node = NormalizedUniqueNode;
    type Precedes = NormalizedUniquePrecedes;
    type Feeds = NormalizedUniqueFeeds;
    type Key = FieldSelection<Self, NormalizedUniqueNode, NodeFacts, u64>;
    type Weight = RelationFieldSelection<
        Self,
        NormalizedUniqueNode,
        NormalizedUniquePrecedes,
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
impl OperationCreates<super::super::super::creation::CreateNode> for NormalizedUniqueNode {}
impl OperationReads<super::super::super::feeding::EditFeeds> for NormalizedUniqueNode {}
impl OperationDeletes<super::super::super::feeding::EditFeeds> for NormalizedUniqueNode {}
impl OperationReads<super::super::super::operation::EditEdges> for NormalizedUniquePrecedes {}
impl OperationLinks<super::super::super::operation::EditEdges> for NormalizedUniquePrecedes {}
impl OperationUnlinks<super::super::super::operation::EditEdges> for NormalizedUniquePrecedes {}
impl OperationLinks<super::super::super::creation::CreateNode> for NormalizedUniquePrecedes {}
impl OperationReads<super::super::super::feeding::EditFeeds> for NormalizedUniqueFeeds {}
impl OperationLinks<super::super::super::feeding::EditFeeds> for NormalizedUniqueFeeds {}
impl OperationUnlinks<super::super::super::feeding::EditFeeds> for NormalizedUniqueFeeds {}
