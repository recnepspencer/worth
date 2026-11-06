//! A separate policy exercises payload birth on a feature-owned outgoing relation.
use super::super::{creation::CreateNode, feeding::EditFeeds, operation::EditEdges};
use super::*;
#[derive(Clone, Debug)]
pub struct ExtensionPayload;
impl LawPolicy for ExtensionPayload {
    type Model = ExtensionRecords;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>
    {
        fragment::declare(GraphSchema::<Self>(PhantomData))
    }
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    const RELATION_PAYLOAD: bool = true;
}
pub struct ExtensionRecords;
impl Fragment for ExtensionRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (
        ExternalMapping,
        Principal,
        Leaf,
        ExtensionNode,
        PayloadRelations,
    );
}
pub struct ExtensionNode;
impl Entity for ExtensionNode {
    const NAME: &'static str = "Node";
    type Aspects = <Node as Entity>::Aspects;
    type Relations = (PayloadFeeds,);
}
pub struct PayloadRelations;
impl Fragment for PayloadRelations {
    const NAME: &'static str = "payload_relations";
    type Bindings = GraphBindings;
    type Members =
        (Extend<ExtensionNode, (), (WithAspects<ExtensionPrecedes, (Option<EdgeFacts>,)>,)>,);
}
pub struct ExtensionPrecedes;
impl Relation for ExtensionPrecedes {
    const NAME: &'static str = "Precedes";
    type To = ExtensionNode;
}
impl GraphModel for ExtensionRecords {
    type Node = ExtensionNode;
    type Precedes = ExtensionPrecedes;
    type Feeds = PayloadFeeds;
    type Key = FieldSelection<Self, ExtensionNode, NodeFacts, u64>;
    type Weight =
        RelationFieldSelection<PayloadRelations, ExtensionNode, ExtensionPrecedes, EdgeFacts, u64>;
    fn node_key() -> Self::Key {
        Self::Key::at("NodeKey")
    }
    fn edge_weight() -> Option<Self::Weight> {
        Some(Self::Weight::at("EdgeWeight"))
    }
}
impl OperationCreates<CreateNode> for ExtensionNode {}
impl OperationReads<EditFeeds> for ExtensionNode {}
impl OperationDeletes<EditFeeds> for ExtensionNode {}
impl OperationReads<EditEdges> for ExtensionPrecedes {}
impl OperationLinks<EditEdges> for ExtensionPrecedes {}
impl OperationUnlinks<EditEdges> for ExtensionPrecedes {}
impl OperationLinks<CreateNode> for ExtensionPrecedes {}
