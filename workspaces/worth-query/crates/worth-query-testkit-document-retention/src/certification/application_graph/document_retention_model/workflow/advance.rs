#[path = "advance/declaration.rs"]
pub mod declaration;
#[path = "advance/mutation.rs"]
pub mod mutation;

pub use declaration::{
    WorkflowAdvanceCapability, WorkflowAdvanceContext, WorkflowAdvanceInput,
    WorkflowAdvanceOperation, WorkflowAdvanceProvenance, WorkflowApprovalCapability,
};
pub use mutation::{
    WorkflowAdvanceBinding, WorkflowAdvanceIntent, WorkflowApprovalBinding, WorkflowApprovalIntent,
};

pub fn install_members(
    schema: worth_query_host::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<
        super::super::schema::DocumentRetentionSchema,
    >,
) -> worth_query_host::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<
    super::super::schema::DocumentRetentionSchema,
> {
    declaration::install_members(schema)
}

pub fn install_binding(
    schema: worth_query_host::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<
        super::super::schema::DocumentRetentionSchema,
    >,
) -> worth_query_host::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<
    super::super::schema::DocumentRetentionSchema,
> {
    mutation::install_binding(schema)
}

pub fn seed(
    graph: &mut worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphBootstrap<
        super::super::schema::DocumentRetentionSchema,
    >,
) {
    declaration::seed(graph);
}
