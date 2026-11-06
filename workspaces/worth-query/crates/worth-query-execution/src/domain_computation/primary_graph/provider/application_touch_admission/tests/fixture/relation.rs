//! Link-only and initialized-create grants remain distinct for record payloads.
use super::*;
use worth_query_decl::facade::application_schema::fragment::{self, *};
pub struct Edge;
impl Relation for Edge {
    const NAME: &'static str = "Edge";
    type To = Node;
}
impl Edge {
    pub fn reference() -> ApplicationRelationRef<TouchSchema, Self, Node, Node> {
        fragment::relation(TouchSchema, Node, Self)
    }
}
pub struct Payload;
impl Aspect for Payload {
    const NAME: &'static str = "Payload";
    type Row = u64;
}
pub(crate) fn weight_field() -> RelationFieldSelection<TouchSchema, Node, Edge, Payload, u64> {
    select_relation_field(TouchSchema, Node, Edge, Payload, "value")
}
worth_query_operation!(pub LinkOnly for TouchSchema, input InputBinding);
worth_query_operation!(pub Initialize for TouchSchema, input InputBinding);
worth_query_operation_reads!(LinkOnly => [Node]);
worth_query_operation_reads!(Initialize => [Node]);
impl OperationLinks<LinkOnly> for Edge {}
impl OperationLinks<Initialize> for Edge {}
impl Registration<TouchSchema> for LinkOnly {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<TouchSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<TouchSchema>, ApplicationSchemaDeclarationDenial>
    {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(operation, 16)
            .operation_read_entity(operation, node())
            .operation_link(operation, Edge::reference()))
    }
}
impl Registration<TouchSchema> for Initialize {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<TouchSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<TouchSchema>, ApplicationSchemaDeclarationDenial>
    {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(operation, 16)
            .operation_read_entity(operation, node())
            .operation_link(operation, Edge::reference())
            .operation_initialize_relation_field(operation, weight_field()))
    }
}
