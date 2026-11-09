//! One relation law model.
use super::super::*;
pub struct DerivedRecords;
impl Fragment for DerivedRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (ExternalMapping, Principal, Leaf, DerivedNode);
}
pub struct DerivedNode;
impl Entity for DerivedNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (DerivedPrecedes, DerivedFeeds);
}
pub struct DerivedPrecedes;
impl Relation for DerivedPrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = DerivedNode;
    const SELF_EDGES: bool = <super::super::super::derived::Derived as LawPolicy>::INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::super::derived::Derived as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::super::derived::Derived as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::super::derived::Derived as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::super::derived::Derived as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool =
        <super::super::super::derived::Derived as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::super::derived::Derived as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::super::derived::Derived as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::super::derived::Derived as LawPolicy>::INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::super::derived::Derived as LawPolicy>::INTEGRITY.reaches_at_least;
}
pub struct DerivedFeeds;
impl Relation for DerivedFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool = <super::super::super::derived::Derived as LawPolicy>::FEEDS_INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::super::derived::Derived as LawPolicy>::FEEDS_INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::super::derived::Derived as LawPolicy>::FEEDS_INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::super::derived::Derived as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool =
        <super::super::super::derived::Derived as LawPolicy>::FEEDS_INTEGRITY.acyclic;
    const SYMMETRIC: bool =
        <super::super::super::derived::Derived as LawPolicy>::FEEDS_INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::super::derived::Derived as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::super::derived::Derived as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::super::derived::Derived as LawPolicy>::FEEDS_INTEGRITY
            .same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::super::derived::Derived as LawPolicy>::FEEDS_INTEGRITY.reaches_at_least;
}

impl GraphModel for DerivedRecords {
    type Node = DerivedNode;
    type Precedes = DerivedPrecedes;
    type Feeds = DerivedFeeds;
    type Key = FieldSelection<Self, DerivedNode, NodeFacts, u64>;
    type Weight = RelationFieldSelection<Self, DerivedNode, DerivedPrecedes, EdgeFacts, u64>;
    fn node_key() -> Self::Key {
        Self::Key::at("NodeKey")
    }
    fn edge_weight() -> Option<Self::Weight> {
        None
    }
}
impl OperationCreates<super::super::super::creation::CreateNode> for DerivedNode {}
impl OperationReads<super::super::super::feeding::EditFeeds> for DerivedNode {}
impl OperationDeletes<super::super::super::feeding::EditFeeds> for DerivedNode {}
impl OperationReads<super::super::super::operation::EditEdges> for DerivedPrecedes {}
impl OperationLinks<super::super::super::operation::EditEdges> for DerivedPrecedes {}
impl OperationUnlinks<super::super::super::operation::EditEdges> for DerivedPrecedes {}
impl OperationLinks<super::super::super::creation::CreateNode> for DerivedPrecedes {}
impl OperationReads<super::super::super::feeding::EditFeeds> for DerivedFeeds {}
impl OperationLinks<super::super::super::feeding::EditFeeds> for DerivedFeeds {}
impl OperationUnlinks<super::super::super::feeding::EditFeeds> for DerivedFeeds {}
