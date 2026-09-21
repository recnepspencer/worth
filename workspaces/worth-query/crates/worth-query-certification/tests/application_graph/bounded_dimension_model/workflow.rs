#[path = "workflow/assessment.rs"]
mod assessment;
#[path = "workflow/contract.rs"]
mod contract;
#[path = "workflow/declaration.rs"]
mod declaration;
#[path = "workflow/definition.rs"]
mod definition;
#[path = "workflow/join_replay_definition.rs"]
mod join_replay_definition;
#[path = "workflow/mutation.rs"]
mod mutation;

pub use advance::{
    WorkflowAdvanceBinding, WorkflowAdvanceCapability, WorkflowAdvanceHandler,
    WorkflowAdvanceInput, WorkflowAdvanceIntent, WorkflowAdvanceOperation, WorkflowApprovalBinding,
    WorkflowApprovalCapability, WorkflowApprovalHandler, WorkflowApprovalIntent,
};
pub use assessment::{accept_assessment, settle_assessment, spoofed_assessment_denial};
pub use declaration::{
    seed_authoring, ReviewedGeometryWorkflow, WorkflowDefinitionAuthoringCapability,
    WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringOperation,
    WorkflowInstanceStartCapability, WorkflowInstanceStartInput, WorkflowInstanceStartOperation,
};
pub use definition::{
    advance_instance, approve_instance, proposal_terminal_definition, propose_instance,
    propose_instance_on_branch, publish_definition, repeated_proposal_definition, retain_workflow,
    reviewed_geometry_definition, start_instance, terminal_definition,
};
pub use join_replay_definition::assessment_join_terminal_definition;
pub use mutation::{
    WorkflowDefinitionAuthoringBinding, WorkflowDefinitionAuthoringHandler,
    WorkflowDefinitionAuthoringIntent, WorkflowInstanceStartBinding, WorkflowInstanceStartHandler,
    WorkflowInstanceStartIntent,
};

pub fn declare(
    schema: worth_query_host::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<
        super::schema::BoundedDimensionSchema,
    >,
) -> worth_query_host::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<
    super::schema::BoundedDimensionSchema,
> {
    advance::install_binding(mutation::install_binding(contract::install(
        advance::install_members(declaration::install_members(schema)),
    )))
}
#[path = "workflow/advance.rs"]
mod advance;
