//! One authored policy composes paired edges with directed physical uniqueness.
use super::{
    fragment, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial, GraphSchema,
    PhantomData, SymmetricDirectedUniqueRecords,
};
pub use super::{ApplicationRelationIntegrity, ApplicationRelationUniquenessScope, LawPolicy};

// compiled-example: begin
pub const PAIRED_UNIQUE: ApplicationRelationIntegrity =
    ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
        .symmetric()
        .unique(ApplicationRelationUniquenessScope::DirectedSemanticEdge);
// compiled-example: end

#[derive(Clone, Debug)]
pub struct SymmetricDirectedUnique;
impl LawPolicy for SymmetricDirectedUnique {
    type Model = SymmetricDirectedUniqueRecords;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>
    {
        fragment::declare(GraphSchema::<Self>(PhantomData))
    }

    const INTEGRITY: ApplicationRelationIntegrity = PAIRED_UNIQUE;
}
