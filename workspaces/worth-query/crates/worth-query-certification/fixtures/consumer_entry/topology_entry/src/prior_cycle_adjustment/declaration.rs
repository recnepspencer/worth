use super::*;
use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

pub(crate) fn declare_prior_cycle_adjustment<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = AdjustPriorCycle::reference::<Schema>();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(operation, 64)
        .operation_projection_work_budget(operation, 16)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_field(operation, PositionY::reference())
        .operation_write(operation, PositionY::reference())
        .application_mutation_binding::<PriorCycleAdjustmentBinding<Schema>>()
}
