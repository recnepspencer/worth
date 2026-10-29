//! One explicit policy adds actual Precedes-owned payload; all old policies stay unchanged.
pub use super::*;
#[derive(Clone, Debug)]
pub struct Payload;
impl LawPolicy for Payload {
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    const RELATION_PAYLOAD: bool = true;
    fn observation_members(
        schema: ApplicationSchemaDeclarationBuilder<GraphSchema<Self>>,
    ) -> ApplicationSchemaDeclarationBuilder<GraphSchema<Self>> {
        super::super::history::declare(schema)
    }
}
worth_query_aspect!(pub EdgeFacts for S: GraphSchemaBinding, Precedes;
    identity = AspectIdentity(0x91811001), revision = AspectContractRevision(1));
pub struct EdgeWeight;
impl DeclaredApplicationFieldValue for EdgeWeight {
    type Value = u64;
    type Binding = U64ApplicationValueBinding;
    const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
}
impl RequiredApplicationFieldValue for EdgeWeight {}
impl<S: GraphSchemaBinding> ApplicationFieldMarkerIdentity<S, Precedes, EdgeFacts> for EdgeWeight {
    const IDENTIFIER: &'static str = "EdgeWeight";
}
impl EdgeWeight {
    pub const fn reference<S: GraphSchemaBinding>(
    ) -> ApplicationRelationFieldRef<S, Precedes, Node, Node, EdgeFacts, Self, ReadWrite> {
        ApplicationRelationFieldRef::from_relation(Precedes::reference())
    }
}
impl OperationWrites<super::super::operation::EditEdges> for EdgeWeight {}
