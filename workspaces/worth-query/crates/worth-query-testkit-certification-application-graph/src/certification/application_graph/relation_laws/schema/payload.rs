//! One explicit policy adds actual Precedes-owned payload; all old policies stay unchanged.
pub use super::*;
#[derive(Clone, Debug)]
pub struct Payload;
impl LawPolicy for Payload {
    type Model = PayloadRecords;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>
    {
        fragment::declare(GraphSchema::<Self>(PhantomData))
    }

    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    const RELATION_PAYLOAD: bool = true;
    fn observation_members(
        schema: ApplicationSchemaDeclarationBuilder<GraphSchema<Self>>,
    ) -> ApplicationSchemaDeclarationBuilder<GraphSchema<Self>> {
        super::super::history::declare(schema)
    }
}
pub struct EdgeFacts;
impl Aspect for EdgeFacts {
    const NAME: &'static str = "EdgeFacts";
    type Row = EdgeRow;
}
pub struct EdgeRow(u64);
impl fragment::Record<EdgeRow> for GraphBindings {
    fn record() -> impl Shape<EdgeRow, Self> {
        field("EdgeWeight", |row: &EdgeRow| row.0)
            .equality_queryable(true)
            .build(|value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(EdgeRow(value))
            })
    }
}
pub type EdgeWeight =
    RelationFieldSelection<PayloadRecords, PayloadNode, PayloadPrecedes, EdgeFacts, u64>;
