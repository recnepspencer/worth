pub use super::*;

// compiled-example: begin
pub const EXPLICIT_CONTEXTS: ApplicationRelationIntegrity = ApplicationRelationIntegrity::new(
    ApplicationRelationEndpoints::new(true, ApplicationRelationCrossContextPolicy::AllowExplicit),
    ApplicationRelationCardinality::unbounded(),
    ApplicationRelationDeletionPolicy::RetainDanglingForAudit,
);
pub const PARTITION_LOCAL: ApplicationRelationIntegrity =
    EXPLICIT_CONTEXTS.same_partition_endpoints();
// compiled-example: end

#[derive(Clone, Debug)]
pub struct CrossPartition;
impl LawPolicy for CrossPartition {
    type Model = CrossPartitionRecords;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>
    {
        fragment::declare(GraphSchema::<Self>(PhantomData))
    }

    const INTEGRITY: ApplicationRelationIntegrity = EXPLICIT_CONTEXTS;
}
#[derive(Clone, Debug)]
pub struct Isolated;
impl LawPolicy for Isolated {
    type Model = IsolatedRecords;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>
    {
        fragment::declare(GraphSchema::<Self>(PhantomData))
    }

    const INTEGRITY: ApplicationRelationIntegrity = PARTITION_LOCAL;
}
