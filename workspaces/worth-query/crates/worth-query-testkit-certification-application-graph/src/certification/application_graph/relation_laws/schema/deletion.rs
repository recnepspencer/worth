//! Live feeding edges protect their actual endpoints from deletion.
use super::*;

#[derive(Clone, Debug)]
pub struct DeletionProtected;
impl LawPolicy for DeletionProtected {
    type Model = DeletionProtectedRecords;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>
    {
        fragment::declare(GraphSchema::<Self>(PhantomData))
    }

    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    const FEEDS_INTEGRITY: ApplicationRelationIntegrity = ApplicationRelationIntegrity {
        cardinality: ApplicationRelationCardinality::new(Some(1), Some(1), None, None, None, None),
        deletion: ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations,
        ..ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
    };
}
