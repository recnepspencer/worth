use super::*;
use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

pub(crate) fn declare_vertex_replacement<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = ReplacePlanarVertex::reference::<Schema>();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(operation, 24)
        .operation_projection_work_budget(operation, 128)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_field(operation, PositionX::reference())
        .operation_read_field(operation, PositionY::reference())
        .operation_read_relation(operation, PlanarSuccessor::reference())
        .operation_create(operation, Body::reference())
        .operation_delete(operation, Body::reference())
        .operation_write(operation, BodyKey::reference())
        .operation_write(operation, PositionX::reference())
        .operation_write(operation, PositionY::reference())
        .operation_write(operation, Length::reference())
        .operation_link(operation, PlanarSuccessor::reference())
        .operation_unlink(operation, PlanarSuccessor::reference())
        .application_mutation_binding::<VertexReplacementBinding<Schema>>()
}
