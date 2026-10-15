#[path = "definition/reviewed.rs"]
mod reviewed;
use reviewed::{reviewed_document_definition_with_policy, Rejection};

use worth_query_host::facade::domain::WorthQueryInstalledWorkflowDefinitionContract;
use worth_query_host::facade::{
    application_entry::{
        PublishedWorkflowDefinitionRef, PublishedWorkflowProposalRef, RequiredWorkflowApproval,
        WorkflowApprovalDecision, WorkflowDefinitionExpectedPredecessor,
        WorkflowDefinitionPublicationOutcome, WorkflowDefinitionRetirementOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
        WorthQueryApplicationRequestExt, WorthQueryWorkflowAdvancePreparationDenial,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
        WorthQueryWorkflowDefinitionRetirementPreparationDenial,
        WorthQueryWorkflowInstancePreparationDenial, WorthQueryWorkflowProposalPreparationDenial,
    },
    declaration::application_program::{
        ApplicationWorkflowComponentLimits, ApplicationWorkflowConditionOperands,
        ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
        ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowDefinitionLimits,
        AuthoredWorkflowDefinition, ValidatedWorkflowDefinition,
    },
};

use super::super::{
    host::DocumentWorkflowRuntime,
    operator_identity::{authenticate_operator, block_on, request_scope},
    retention_days::DocumentRetentionDaysQuery,
    retention_entry::{ReviewedSetRetentionBinding, DOCUMENT_IDENTITY},
    schema::{DocumentRetentionConditionQuery, DocumentRetentionQuery, DocumentRetentionSchema},
};
use super::{
    ReviewedDocumentWorkflow, WorkflowAdvanceInput, WorkflowAdvanceIntent,
    WorkflowApprovalCapability, WorkflowApprovalIntent, WorkflowDefinitionAuthoringInput,
    WorkflowDefinitionAuthoringIntent, WorkflowDefinitionAuthoringOperation,
    WorkflowInstanceStartInput, WorkflowInstanceStartIntent,
};

#[path = "definition/authoring.rs"]
mod authoring;
pub use authoring::{authoring_intent, publish_definition, retire_definition};
mod discovery;
#[path = "definition/instance.rs"]
mod instance;
pub use discovery::discover_definition_metadata;
pub use instance::{
    advance_instance, advance_instance_on, approve_instance, cancel_instance, continue_on_fork,
    migrate_instance, prepare_cancellation, prepare_cancellation_on, propose_authoring_instance,
    propose_authoring_instance_with_retention, propose_instance, propose_instance_on_branch,
    start_instance,
};

pub fn reviewed_document_definition(
    completion_identity: &str,
) -> ValidatedWorkflowDefinition<ReviewedDocumentWorkflow> {
    reviewed_document_definition_with_join_policy(
        completion_identity,
        worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
    )
}

pub fn reviewed_document_definition_with_join_policy(
    completion_identity: &str,
    join_policy: worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy,
) -> ValidatedWorkflowDefinition<ReviewedDocumentWorkflow> {
    reviewed_document_definition_with_policy::<ReviewedSetRetentionBinding>(
        completion_identity,
        join_policy,
        Rejection::Terminal,
        definition_limits(),
    )
}

/// Every instance of this definition must finish within `deadline` of its
/// start.
pub fn reviewed_document_definition_with_deadline(
    completion_identity: &str,
    deadline: std::time::Duration,
) -> ValidatedWorkflowDefinition<ReviewedDocumentWorkflow> {
    reviewed_document_definition_with_policy::<ReviewedSetRetentionBinding>(
        completion_identity,
        worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        Rejection::Terminal,
        definition_limits()
            .with_total_deadline(deadline)
            .expect("the deadline is whole nonzero milliseconds"),
    )
}

/// A rejected approval loops back to a fresh proposal.
pub fn reproposing_document_definition(
    completion_identity: &str,
) -> ValidatedWorkflowDefinition<ReviewedDocumentWorkflow> {
    reviewed_document_definition_with_policy::<ReviewedSetRetentionBinding>(
        completion_identity,
        worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        Rejection::Repropose,
        definition_limits(),
    )
}

pub fn reviewed_document_definition_for_binding<
    Binding: worth_query_host::facade::declaration::application_operation::ApplicationMutationBinding<
        DocumentRetentionSchema,
    >,
>(
    completion_identity: &str,
) -> ValidatedWorkflowDefinition<ReviewedDocumentWorkflow> {
    reviewed_document_definition_with_policy::<Binding>(
        completion_identity,
        worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        Rejection::Terminal, definition_limits(),
    )
}

pub fn approval_retry_definition() -> ValidatedWorkflowDefinition<ReviewedDocumentWorkflow> {
    reviewed_document_definition_with_policy::<ReviewedSetRetentionBinding>(
        "applied",
        worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        Rejection::Retry(2),
        definition_limits(),
    )
}

pub fn terminal_definition(
    terminal_identity: &str,
) -> ValidatedWorkflowDefinition<ReviewedDocumentWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedDocumentWorkflow>::new(
        "reviewed-document-terminal",
        definition_limits(),
    )
    .expect("the terminal workflow identity is valid");
    let terminal = builder
        .terminal(terminal_identity)
        .expect("the terminal start node is valid");
    builder.start(&terminal);
    builder
        .finish()
        .expect("the terminal definition is complete")
        .validate()
        .expect("the terminal definition is valid")
}

pub fn proposal_terminal_definition() -> ValidatedWorkflowDefinition<ReviewedDocumentWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedDocumentWorkflow>::new(
        "proposal-terminal",
        definition_limits(),
    )
    .expect("the proposal-terminal workflow identity is valid");
    let proposal = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("proposal", false)
        .expect("the proposal node is valid");
    let terminal = builder
        .terminal("completed")
        .expect("the terminal node is valid");
    builder.start(&proposal).control(
        &proposal,
        ApplicationWorkflowControlOutcome::Completed,
        &terminal,
    );
    builder
        .finish()
        .expect("the proposal-terminal definition is complete")
        .validate()
        .expect("the proposal-terminal definition is valid")
}

pub fn condition_terminal_definition() -> ValidatedWorkflowDefinition<ReviewedDocumentWorkflow> {
    condition_terminal_draft()
        .validate()
        .expect("the condition definition is valid")
}

pub fn condition_terminal_draft() -> AuthoredWorkflowDefinition<ReviewedDocumentWorkflow> {
    condition_draft(
        "retained",
        ApplicationWorkflowConditionOperands::new()
            .query::<DocumentRetentionConditionQuery>("retained"),
    )
}

/// The condition terminal deciding by `source` over the Bool read `retained`
/// and the unsigned day count `days` of the same document.
pub fn expression_condition_terminal_definition(
    source: &str,
) -> ValidatedWorkflowDefinition<ReviewedDocumentWorkflow> {
    condition_draft(
        source,
        ApplicationWorkflowConditionOperands::new()
            .query::<DocumentRetentionConditionQuery>("retained")
            .query::<DocumentRetentionDaysQuery>("days"),
    )
    .validate()
    .expect("the expression condition definition is valid")
}

fn condition_draft(
    source: &str,
    operands: ApplicationWorkflowConditionOperands<ReviewedDocumentWorkflow>,
) -> AuthoredWorkflowDefinition<ReviewedDocumentWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedDocumentWorkflow>::new(
        "condition-terminal",
        definition_limits(),
    )
    .expect("the condition workflow identity is valid");
    let proposal = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("proposal", false)
        .expect("the proposal node is valid");
    let condition = builder
        .condition("positive-retention", source, operands)
        .expect("the condition node is valid");
    let satisfied = builder
        .terminal("satisfied")
        .expect("the satisfied terminal is valid");
    let unsatisfied = builder
        .terminal("unsatisfied")
        .expect("the unsatisfied terminal is valid");
    builder
        .start(&proposal)
        .control(
            &proposal,
            ApplicationWorkflowControlOutcome::Completed,
            &condition,
        )
        .control(
            &condition,
            ApplicationWorkflowControlOutcome::ConditionSatisfied,
            &satisfied,
        )
        .control(
            &condition,
            ApplicationWorkflowControlOutcome::ConditionUnsatisfied,
            &unsatisfied,
        )
        .condition_subject(&proposal, &condition);
    builder
        .finish()
        .expect("the condition definition is complete")
}

pub fn repeated_proposal_definition() -> ValidatedWorkflowDefinition<ReviewedDocumentWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedDocumentWorkflow>::new(
        "repeated-proposal",
        definition_limits(),
    )
    .expect("the repeated-proposal workflow identity is valid");
    let first = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("proposal/first", false)
        .expect("the first proposal node is valid");
    let second = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("proposal/second", false)
        .expect("the second proposal node is valid");
    let terminal = builder
        .terminal("completed")
        .expect("the terminal node is valid");
    builder
        .start(&first)
        .control(
            &first,
            ApplicationWorkflowControlOutcome::Completed,
            &second,
        )
        .control(
            &second,
            ApplicationWorkflowControlOutcome::Completed,
            &terminal,
        );
    builder
        .finish()
        .expect("the repeated-proposal definition is complete")
        .validate()
        .expect("the repeated-proposal definition is valid")
}

pub fn bind_definition(
    application: &DocumentWorkflowRuntime,
    definition: ValidatedWorkflowDefinition<ReviewedDocumentWorkflow>,
) -> WorthQueryInstalledWorkflowDefinitionContract<DocumentRetentionSchema, ReviewedDocumentWorkflow>
{
    application
        .workflow_spec()
        .bind_definition(definition)
        .expect("the workflow definition binds to installed vocabulary")
}

pub(crate) fn definition_limits() -> ApplicationWorkflowDefinitionLimits {
    ApplicationWorkflowDefinitionLimits::new(
        32,
        64,
        4,
        ApplicationWorkflowComponentLimits::new(32, 4, 128, 256, 256).unwrap(),
        64 * 1024,
    )
    .expect("the workflow definition limits are nonzero")
}
