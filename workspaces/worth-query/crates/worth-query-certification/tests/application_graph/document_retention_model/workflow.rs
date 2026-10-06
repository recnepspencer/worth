#[path = "workflow/assessment.rs"]
mod assessment;
#[path = "workflow/authentication.rs"]
mod authentication;
#[path = "workflow/contract.rs"]
mod contract;
#[path = "workflow/declaration.rs"]
mod declaration;
#[path = "workflow/definition.rs"]
mod definition;
#[path = "workflow/grant_status.rs"]
mod grant_status;
#[path = "workflow/join_replay_definition.rs"]
mod join_replay_definition;
#[path = "workflow/mutation.rs"]
mod mutation;
#[path = "workflow/ordinary_run.rs"]
mod ordinary_run;
#[path = "workflow/program_adoption.rs"]
mod program_adoption;
#[path = "workflow/retry_definition.rs"]
mod retry_definition;
#[path = "workflow/review_requirement.rs"]
mod review_requirement;
#[path = "workflow/runtime.rs"]
mod runtime;
#[path = "workflow/start_outcome.rs"]
mod start_outcome;

pub use advance::{
    WorkflowAdvanceBinding, WorkflowAdvanceCapability, WorkflowAdvanceInput, WorkflowAdvanceIntent,
    WorkflowAdvanceOperation, WorkflowApprovalBinding, WorkflowApprovalCapability,
    WorkflowApprovalIntent,
};
pub use assessment::{
    accept_assessment, accept_early_assessment, prepare_early_assessment_denial, settle_assessment,
    settle_assessment_for, settle_early_assessment_for, spoofed_assessment_denial,
};
pub use authentication::{
    install_certification_authentication, install_certification_authentication_with_clock,
    CertificationAuthenticationClockSource, CertificationAuthenticationOwner,
};
pub use declaration::{
    seed_authoring, ReviewedDocumentWorkflow, WorkflowDefinitionAuthoringCapability,
    WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringOperation,
    WorkflowInstanceStartCapability, WorkflowInstanceStartInput, WorkflowInstanceStartOperation,
};
pub(crate) use definition::definition_limits;
pub use definition::{
    advance_instance, advance_instance_on, approval_retry_definition, approve_instance,
    authoring_intent, cancel_instance, condition_terminal_definition, condition_terminal_draft,
    continue_on_fork, expression_condition_terminal_definition, migrate_instance,
    prepare_cancellation, prepare_cancellation_on, proposal_terminal_definition,
    propose_authoring_instance, propose_authoring_instance_with_retention, propose_instance,
    propose_instance_on_branch, publish_definition, repeated_proposal_definition,
    reproposing_document_definition, retire_definition, reviewed_document_definition,
    reviewed_document_definition_with_deadline, reviewed_document_definition_with_join_policy,
    start_instance, terminal_definition,
};
pub use grant_status::{
    WorkflowGrantStatusBinding, WorkflowGrantStatusHandler, WorkflowGrantStatusInput,
    WorkflowGrantStatusIntent,
};
pub use join_replay_definition::{
    assessment_join_terminal_definition, assessment_join_terminal_definition_with_policy,
    assessment_retry_definition, conditionally_required_related_assessment_definition,
    early_assessment_definition, multi_subject_assessment_retry_definition,
};
pub use mutation::{
    WorkflowDefinitionAuthoringBinding, WorkflowDefinitionAuthoringIntent,
    WorkflowInstanceStartBinding, WorkflowInstanceStartIntent,
};
pub use ordinary_run::run_instance;
pub use program_adoption::{
    prepare_second_program_adoption, publish_adoption, second_program_workflow_inventory,
    second_revision,
};
pub use retry_definition::{bounded_retry_definition, bounded_retry_definition_with_attempts};
pub use review_requirement::{
    link_review_requirement, link_review_requirement_on, unlink_review_requirement,
    unlink_review_requirement_on, ReviewRequirementBinding, ReviewRequirementDenial,
    ReviewRequirementHandler, ReviewRequirementInput, UnlinkReviewRequirementBinding,
    UnlinkReviewRequirementHandler,
};
pub use runtime::{
    install_workflow_spec, retain_workflow, retain_workflow_with_resources,
    support_workflow_program,
};
pub use start_outcome::{expect_capacity_refused, expect_retired_start, expect_superseded_start};

pub fn declare(
    schema: worth_query_host::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<
        super::schema::DocumentRetentionSchema,
    >,
) -> worth_query_host::facade::declaration::application_schema::ApplicationSchemaDeclarationBuilder<
    super::schema::DocumentRetentionSchema,
> {
    advance::install_binding(mutation::install_binding(contract::install(
        grant_status::install_members(advance::install_members(declaration::install_members(
            review_requirement::declare(schema),
        ))),
    )))
}
#[path = "workflow/advance.rs"]
mod advance;
