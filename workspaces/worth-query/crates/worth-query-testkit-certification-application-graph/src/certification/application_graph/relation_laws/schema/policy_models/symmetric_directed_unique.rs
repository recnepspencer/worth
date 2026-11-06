//! One relation law model.
use super::super::*;
pub struct SymmetricDirectedUniqueRecords;
impl Fragment for SymmetricDirectedUniqueRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (
        ExternalMapping,
        Principal,
        Leaf,
        SymmetricDirectedUniqueNode,
    );
}
pub struct SymmetricDirectedUniqueNode;
impl Entity for SymmetricDirectedUniqueNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (
        SymmetricDirectedUniquePrecedes,
        SymmetricDirectedUniqueFeeds,
    );
}
pub struct SymmetricDirectedUniquePrecedes;
impl Relation for SymmetricDirectedUniquePrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = SymmetricDirectedUniqueNode;
    const SELF_EDGES: bool =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::INTEGRITY
            .endpoints
            .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::INTEGRITY
            .same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::INTEGRITY
            .reaches_at_least;
}
pub struct SymmetricDirectedUniqueFeeds;
impl Relation for SymmetricDirectedUniqueFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::FEEDS_INTEGRITY
            .cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::FEEDS_INTEGRITY
            .deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::FEEDS_INTEGRITY
            .acyclic;
    const SYMMETRIC: bool =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::FEEDS_INTEGRITY
            .symmetric;
    const ORBIT: bool =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::FEEDS_INTEGRITY
            .same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::paired_unique::SymmetricDirectedUnique as LawPolicy>::FEEDS_INTEGRITY
            .reaches_at_least;
}

impl GraphModel for SymmetricDirectedUniqueRecords {
    type Node = SymmetricDirectedUniqueNode;
    type Precedes = SymmetricDirectedUniquePrecedes;
    type Feeds = SymmetricDirectedUniqueFeeds;
    type Key = FieldSelection<Self, SymmetricDirectedUniqueNode, NodeFacts, u64>;
    type Weight = RelationFieldSelection<
        Self,
        SymmetricDirectedUniqueNode,
        SymmetricDirectedUniquePrecedes,
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
impl OperationCreates<super::super::super::creation::CreateNode> for SymmetricDirectedUniqueNode {}
impl OperationReads<super::super::super::feeding::EditFeeds> for SymmetricDirectedUniqueNode {}
impl OperationDeletes<super::super::super::feeding::EditFeeds> for SymmetricDirectedUniqueNode {}
impl OperationReads<super::super::super::operation::EditEdges> for SymmetricDirectedUniquePrecedes {}
impl OperationLinks<super::super::super::operation::EditEdges> for SymmetricDirectedUniquePrecedes {}
impl OperationUnlinks<super::super::super::operation::EditEdges>
    for SymmetricDirectedUniquePrecedes
{
}
impl OperationLinks<super::super::super::creation::CreateNode> for SymmetricDirectedUniquePrecedes {}
impl OperationReads<super::super::super::feeding::EditFeeds> for SymmetricDirectedUniqueFeeds {}
impl OperationLinks<super::super::super::feeding::EditFeeds> for SymmetricDirectedUniqueFeeds {}
impl OperationUnlinks<super::super::super::feeding::EditFeeds> for SymmetricDirectedUniqueFeeds {}
