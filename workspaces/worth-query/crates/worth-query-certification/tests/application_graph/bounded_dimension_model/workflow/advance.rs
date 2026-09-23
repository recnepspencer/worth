#[path = "advance/declaration.rs"]
mod declaration;
#[path = "advance/mutation.rs"]
mod mutation;

pub use declaration::{
    WorkflowAdvanceCapability, WorkflowAdvanceContext, WorkflowAdvanceInput,
    WorkflowAdvanceOperation, WorkflowAdvanceProvenance, WorkflowApprovalCapability,
};
pub use mutation::{
    WorkflowAdvanceBinding, WorkflowAdvanceHandler, WorkflowAdvanceIntent, WorkflowApprovalBinding,
    WorkflowApprovalHandler, WorkflowApprovalIntent,
};

pub(super) fn install_members(
    schema: worth_query_host::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<
        super::super::schema::BoundedDimensionSchema,
    >,
) -> worth_query_host::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<
    super::super::schema::BoundedDimensionSchema,
> {
    declaration::install_members(schema)
}

pub(super) fn install_binding(
    schema: worth_query_host::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<
        super::super::schema::BoundedDimensionSchema,
    >,
) -> worth_query_host::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<
    super::super::schema::BoundedDimensionSchema,
> {
    mutation::install_binding(schema)
}

pub(super) fn seed(
    graph: &mut worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphBootstrap<
        super::super::schema::BoundedDimensionSchema,
    >,
) {
    declaration::seed(graph);
}
