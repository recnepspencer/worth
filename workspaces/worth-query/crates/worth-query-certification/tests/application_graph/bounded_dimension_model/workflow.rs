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
#[path = "workflow/retry_definition.rs"]
mod retry_definition;
#[path = "workflow/runtime.rs"]
mod runtime;

pub use advance::{
    WorkflowAdvanceBinding, WorkflowAdvanceCapability, WorkflowAdvanceHandler,
    WorkflowAdvanceInput, WorkflowAdvanceIntent, WorkflowAdvanceOperation, WorkflowApprovalBinding,
    WorkflowApprovalCapability, WorkflowApprovalHandler, WorkflowApprovalIntent,
};
pub use assessment::{
    accept_assessment, settle_assessment, settle_assessment_for, spoofed_assessment_denial,
};
pub use declaration::{
    seed_authoring, ReviewedGeometryWorkflow, WorkflowDefinitionAuthoringCapability,
    WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringOperation,
    WorkflowInstanceStartCapability, WorkflowInstanceStartInput, WorkflowInstanceStartOperation,
};
pub(crate) use definition::definition_limits;
pub use definition::{
    advance_instance, approval_retry_definition, approve_instance, condition_terminal_definition,
    proposal_terminal_definition, propose_authoring_instance, propose_instance,
    propose_instance_on_branch, publish_definition, repeated_proposal_definition,
    reviewed_geometry_definition, reviewed_geometry_definition_with_join_policy, start_instance,
    terminal_definition,
};
pub use join_replay_definition::{
    assessment_join_terminal_definition, assessment_join_terminal_definition_with_policy,
    assessment_retry_definition, multi_subject_assessment_retry_definition,
};
pub use mutation::{
    WorkflowDefinitionAuthoringBinding, WorkflowDefinitionAuthoringHandler,
    WorkflowDefinitionAuthoringIntent, WorkflowInstanceStartBinding, WorkflowInstanceStartHandler,
    WorkflowInstanceStartIntent,
};
pub use retry_definition::{bounded_retry_definition, bounded_retry_definition_with_attempts};
pub use runtime::{retain_workflow, retain_workflow_with_resources};

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
