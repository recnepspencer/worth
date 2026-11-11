//! One relation law model.
use super::super::*;
pub struct PayloadRecords;
impl Fragment for PayloadRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (ExternalMapping, Principal, Leaf, PayloadNode);
}
pub struct PayloadNode;
impl Entity for PayloadNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (
        WithAspects<PayloadPrecedes, (Option<EdgeFacts>,)>,
        PayloadFeeds,
    );
}
pub struct PayloadPrecedes;
impl Relation for PayloadPrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = PayloadNode;
    const SELF_EDGES: bool = <super::super::payload::Payload as LawPolicy>::INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::payload::Payload as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::payload::Payload as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::payload::Payload as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::payload::Payload as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool = <super::super::payload::Payload as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::payload::Payload as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::payload::Payload as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::payload::Payload as LawPolicy>::INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::payload::Payload as LawPolicy>::INTEGRITY.reaches_at_least;
}
pub struct PayloadFeeds;
impl Relation for PayloadFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool = <super::super::payload::Payload as LawPolicy>::FEEDS_INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <super::super::payload::Payload as LawPolicy>::FEEDS_INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <super::super::payload::Payload as LawPolicy>::FEEDS_INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <super::super::payload::Payload as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <super::super::payload::Payload as LawPolicy>::FEEDS_INTEGRITY.acyclic;
    const SYMMETRIC: bool =
        <super::super::payload::Payload as LawPolicy>::FEEDS_INTEGRITY.symmetric;
    const ORBIT: bool = <super::super::payload::Payload as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <super::super::payload::Payload as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <super::super::payload::Payload as LawPolicy>::FEEDS_INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <super::super::payload::Payload as LawPolicy>::FEEDS_INTEGRITY.reaches_at_least;
}

impl GraphModel for PayloadRecords {
    type Node = PayloadNode;
    type Precedes = PayloadPrecedes;
    type Feeds = PayloadFeeds;
    type Key = FieldSelection<Self, PayloadNode, NodeFacts, u64>;
    type Weight = RelationFieldSelection<Self, PayloadNode, PayloadPrecedes, EdgeFacts, u64>;
    fn node_key() -> Self::Key {
        Self::Key::at("NodeKey")
    }
    fn edge_weight() -> Option<Self::Weight> {
        Some(Self::Weight::at("EdgeWeight"))
    }
}
impl OperationCreates<super::super::super::creation::CreateNode> for PayloadNode {}
impl OperationReads<super::super::super::feeding::EditFeeds> for PayloadNode {}
impl OperationDeletes<super::super::super::feeding::EditFeeds> for PayloadNode {}
impl OperationReads<super::super::super::operation::EditEdges> for PayloadPrecedes {}
impl OperationLinks<super::super::super::operation::EditEdges> for PayloadPrecedes {}
impl OperationUnlinks<super::super::super::operation::EditEdges> for PayloadPrecedes {}
impl OperationLinks<super::super::super::creation::CreateNode> for PayloadPrecedes {}
impl OperationReads<super::super::super::feeding::EditFeeds> for PayloadFeeds {}
impl OperationLinks<super::super::super::feeding::EditFeeds> for PayloadFeeds {}
impl OperationUnlinks<super::super::super::feeding::EditFeeds> for PayloadFeeds {}
