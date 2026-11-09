//! One relation law model.
use super::*;
pub struct RepeatedRecords;
impl Fragment for RepeatedRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (ExternalMapping, Principal, Leaf, RepeatedNode);
}
pub struct RepeatedNode;
impl Entity for RepeatedNode {
    const NAME: &'static str = <Node as Entity>::NAME;
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (RepeatedPrecedes, RepeatedFeeds);
}
pub struct RepeatedPrecedes;
impl Relation for RepeatedPrecedes {
    const NAME: &'static str = <Precedes as Relation>::NAME;
    type To = RepeatedNode;
    const SELF_EDGES: bool = <RepeatedKinds as LawPolicy>::INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <RepeatedKinds as LawPolicy>::INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <RepeatedKinds as LawPolicy>::INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <RepeatedKinds as LawPolicy>::INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <RepeatedKinds as LawPolicy>::INTEGRITY.acyclic;
    const SYMMETRIC: bool = <RepeatedKinds as LawPolicy>::INTEGRITY.symmetric;
    const ORBIT: bool = <RepeatedKinds as LawPolicy>::INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <RepeatedKinds as LawPolicy>::INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <RepeatedKinds as LawPolicy>::INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <RepeatedKinds as LawPolicy>::INTEGRITY.reaches_at_least;
}
pub struct RepeatedFeeds;
impl Relation for RepeatedFeeds {
    const NAME: &'static str = <Feeds as Relation>::NAME;
    type To = Leaf;
    const SELF_EDGES: bool = <RepeatedKinds as LawPolicy>::FEEDS_INTEGRITY
        .endpoints
        .self_edges_allowed;
    const CARDINALITY: ApplicationRelationCardinality =
        <RepeatedKinds as LawPolicy>::FEEDS_INTEGRITY.cardinality;
    const DELETION: ApplicationRelationDeletionPolicy =
        <RepeatedKinds as LawPolicy>::FEEDS_INTEGRITY.deletion;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        <RepeatedKinds as LawPolicy>::FEEDS_INTEGRITY
            .endpoints
            .cross_context_policy;
    const ACYCLIC: bool = <RepeatedKinds as LawPolicy>::FEEDS_INTEGRITY.acyclic;
    const SYMMETRIC: bool = <RepeatedKinds as LawPolicy>::FEEDS_INTEGRITY.symmetric;
    const ORBIT: bool = <RepeatedKinds as LawPolicy>::FEEDS_INTEGRITY.orbit;
    const UNIQUE: Option<ApplicationRelationUniquenessScope> =
        <RepeatedKinds as LawPolicy>::FEEDS_INTEGRITY.unique;
    const SAME_PARTITION_ENDPOINTS: bool =
        <RepeatedKinds as LawPolicy>::FEEDS_INTEGRITY.same_partition_endpoints;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        <RepeatedKinds as LawPolicy>::FEEDS_INTEGRITY.reaches_at_least;
}

impl GraphModel for RepeatedRecords {
    type Node = RepeatedNode;
    type Precedes = RepeatedPrecedes;
    type Feeds = RepeatedFeeds;
    type Key = FieldSelection<Self, RepeatedNode, NodeFacts, u64>;
    type Weight = RelationFieldSelection<Self, RepeatedNode, RepeatedPrecedes, EdgeFacts, u64>;
    fn node_key() -> Self::Key {
        Self::Key::at("NodeKey")
    }
    fn edge_weight() -> Option<Self::Weight> {
        None
    }
}
impl OperationCreates<worth_query_testkit_certification_application_graph::facade::certification::relation_laws::creation::CreateNode> for RepeatedNode {}
impl OperationReads<worth_query_testkit_certification_application_graph::facade::certification::relation_laws::feeding::EditFeeds> for RepeatedNode {}
impl OperationDeletes<worth_query_testkit_certification_application_graph::facade::certification::relation_laws::feeding::EditFeeds> for RepeatedNode {}
impl OperationReads<worth_query_testkit_certification_application_graph::facade::certification::relation_laws::operation::EditEdges> for RepeatedPrecedes {}
impl OperationLinks<worth_query_testkit_certification_application_graph::facade::certification::relation_laws::operation::EditEdges> for RepeatedPrecedes {}
impl OperationUnlinks<worth_query_testkit_certification_application_graph::facade::certification::relation_laws::operation::EditEdges> for RepeatedPrecedes {}
impl OperationLinks<worth_query_testkit_certification_application_graph::facade::certification::relation_laws::creation::CreateNode> for RepeatedPrecedes {}
impl OperationReads<worth_query_testkit_certification_application_graph::facade::certification::relation_laws::feeding::EditFeeds> for RepeatedFeeds {}
impl OperationLinks<worth_query_testkit_certification_application_graph::facade::certification::relation_laws::feeding::EditFeeds> for RepeatedFeeds {}
impl OperationUnlinks<worth_query_testkit_certification_application_graph::facade::certification::relation_laws::feeding::EditFeeds> for RepeatedFeeds {}

#[derive(Clone, Debug)]
pub struct RepeatedKinds;
impl LawPolicy for RepeatedKinds {
    type Model = RepeatedRecords;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>
    {
        fragment::declare(GraphSchema::<Self>(PhantomData))
    }
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
            .reaches_at_least(2, &["Node", "Node"]);
}
