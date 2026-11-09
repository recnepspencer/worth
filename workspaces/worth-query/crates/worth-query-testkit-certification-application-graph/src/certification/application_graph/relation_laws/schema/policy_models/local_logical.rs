//! One relation law model.
use super::super::*;
pub struct LocalLogicalRecords;
impl Fragment for LocalLogicalRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (ExternalMapping, Principal, Leaf, LocalLogicalNode);
}
pub struct LocalLogicalNode;
impl Entity for LocalLogicalNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (LocalLogicalPrecedes, LocalLogicalFeeds);
}
pub struct LocalLogicalPrecedes;
impl Relation for LocalLogicalPrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = LocalLogicalNode;
    const SELF_EDGES: bool =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::INTEGRITY
            .endpoints
            .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::INTEGRITY
            .same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::INTEGRITY
            .reaches_at_least;
}
pub struct LocalLogicalFeeds;
impl Relation for LocalLogicalFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::FEEDS_INTEGRITY
            .cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::FEEDS_INTEGRITY
            .deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::FEEDS_INTEGRITY
            .acyclic;
    const SYMMETRIC: bool =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::FEEDS_INTEGRITY
            .symmetric;
    const ORBIT: bool =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::FEEDS_INTEGRITY
            .same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::super::logical::locality::LocalLogical as LawPolicy>::FEEDS_INTEGRITY
            .reaches_at_least;
}

impl GraphModel for LocalLogicalRecords {
    type Node = LocalLogicalNode;
    type Precedes = LocalLogicalPrecedes;
    type Feeds = LocalLogicalFeeds;
    type Key = FieldSelection<Self, LocalLogicalNode, NodeFacts, u64>;
    type Weight =
        RelationFieldSelection<Self, LocalLogicalNode, LocalLogicalPrecedes, EdgeFacts, u64>;
    fn node_key() -> Self::Key {
        Self::Key::at("NodeKey")
    }
    fn edge_weight() -> Option<Self::Weight> {
        None
    }
}
impl OperationCreates<super::super::super::creation::CreateNode> for LocalLogicalNode {}
impl OperationReads<super::super::super::feeding::EditFeeds> for LocalLogicalNode {}
impl OperationDeletes<super::super::super::feeding::EditFeeds> for LocalLogicalNode {}
impl OperationReads<super::super::super::operation::EditEdges> for LocalLogicalPrecedes {}
impl OperationLinks<super::super::super::operation::EditEdges> for LocalLogicalPrecedes {}
impl OperationUnlinks<super::super::super::operation::EditEdges> for LocalLogicalPrecedes {}
impl OperationLinks<super::super::super::creation::CreateNode> for LocalLogicalPrecedes {}
impl OperationReads<super::super::super::feeding::EditFeeds> for LocalLogicalFeeds {}
impl OperationLinks<super::super::super::feeding::EditFeeds> for LocalLogicalFeeds {}
impl OperationUnlinks<super::super::super::feeding::EditFeeds> for LocalLogicalFeeds {}
