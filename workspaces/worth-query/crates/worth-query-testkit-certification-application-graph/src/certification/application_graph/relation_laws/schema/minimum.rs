pub use super::*;
// compiled-example: begin
pub const MINIMUM_TWO: ApplicationRelationIntegrity =
    ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
        .reaches_at_least(2, &["Node"]);
// compiled-example: end

#[derive(Clone, Debug)]
pub struct MinimumTwo;
impl LawPolicy for MinimumTwo {
    type Model = MinimumTwoRecords;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>
    {
        fragment::declare(GraphSchema::<Self>(PhantomData))
    }

    const INTEGRITY: ApplicationRelationIntegrity = MINIMUM_TWO;
}
#[derive(Clone, Debug)]
pub struct LeafMinimumTwo;
impl LawPolicy for LeafMinimumTwo {
    type Model = LeafMinimumTwoRecords;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>
    {
        fragment::declare(GraphSchema::<Self>(PhantomData))
    }

    const INTEGRITY: ApplicationRelationIntegrity = Unrestricted::INTEGRITY;
    const FEEDS_INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
            .reaches_at_least(2, &[Leaf::reference::<GraphSchema<Self>>().name()]);
}

#[derive(Clone, Debug)]
pub struct LeafAndPrincipalMinimumTwo;
impl LawPolicy for LeafAndPrincipalMinimumTwo {
    type Model = LeafAndPrincipalMinimumTwoRecords;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>
    {
        fragment::declare(GraphSchema::<Self>(PhantomData))
    }

    const INTEGRITY: ApplicationRelationIntegrity = Unrestricted::INTEGRITY;
    const FEEDS_INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
            .reaches_at_least(2, &["Principal", "Leaf", "Leaf"]);
}
