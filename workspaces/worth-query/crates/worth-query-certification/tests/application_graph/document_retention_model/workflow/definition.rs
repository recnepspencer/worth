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
        ApplicationWorkflowDefinitionLimits, AuthoredWorkflowDefinition,
        ValidatedWorkflowDefinition,
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
#[path = "definition/instance.rs"]
mod instance;
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
    reviewed_document_definition_with_policy(
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
    reviewed_document_definition_with_policy(
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
    reviewed_document_definition_with_policy(
        completion_identity,
        worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        Rejection::Repropose,
        definition_limits(),
    )
}

enum Rejection {
    Terminal,
    Retry(u16),
    Repropose,
}

pub fn approval_retry_definition() -> ValidatedWorkflowDefinition<ReviewedDocumentWorkflow> {
    reviewed_document_definition_with_policy(
        "applied",
        worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        Rejection::Retry(2),
        definition_limits(),
    )
}

fn reviewed_document_definition_with_policy(
    completion_identity: &str,
    join_policy: worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy,
    rejection: Rejection,
    limits: ApplicationWorkflowDefinitionLimits,
) -> ValidatedWorkflowDefinition<ReviewedDocumentWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedDocumentWorkflow>::new(
        "reviewed-document",
        limits,
    )
    .expect("the workflow identity is valid");
    let propose = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .expect("the proposal node is valid");
    let structural = builder
        .assessment::<DocumentRetentionQuery>("checks/structural")
        .expect("the structural assessment node is valid");
    let compliance = builder
        .assessment::<DocumentRetentionQuery>("checks/compliance")
        .expect("the compliance assessment node is valid");
    let evidence = builder
        .evidence_join("checks/evidence", join_policy)
        .expect("the evidence node is valid");
    let approval = builder
        .approval::<WorkflowApprovalCapability>("approval")
        .expect("the approval node is valid");
    let apply = builder
        .operation_binding::<ReviewedSetRetentionBinding>("apply")
        .expect("the guarded operation node is valid");
    let completed = builder
        .terminal(completion_identity)
        .expect("the completion node is valid");
    let rejected = builder
        .terminal("rejected")
        .expect("the rejection node is valid");
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &structural,
        )
        .control(
            &structural,
            ApplicationWorkflowControlOutcome::Completed,
            &compliance,
        )
        .control(
            &compliance,
            ApplicationWorkflowControlOutcome::Completed,
            &evidence,
        )
        .control(
            &evidence,
            ApplicationWorkflowControlOutcome::EvidenceSatisfied,
            &approval,
        )
        .control(
            &evidence,
            ApplicationWorkflowControlOutcome::EvidenceFailed,
            &rejected,
        )
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::Approved,
            &apply,
        )
        .control(
            &apply,
            ApplicationWorkflowControlOutcome::Completed,
            &completed,
        )
        .proposal_for_assessment(&propose, &structural)
        .proposal_for_assessment(&propose, &compliance)
        .assessment_evidence(&structural, &evidence)
        .assessment_evidence(&compliance, &evidence)
        .proposal_for_approval(&propose, &approval)
        .joined_evidence(&evidence, &approval)
        .approval_authority(&approval, &apply)
        .operation_input(&propose, &apply);
    match rejection {
        Rejection::Retry(bound) => {
            builder.retry(
            &approval,
            worth_query_host::facade::declaration::application_program::ApplicationWorkflowRetry::new(
                ApplicationWorkflowControlOutcome::Rejected, "reconsider", bound,
            ).expect("the reconsideration bound is valid"),
            &approval,
        ).control(&approval, ApplicationWorkflowControlOutcome::RetryExhausted, &rejected);
        }
        Rejection::Terminal => {
            builder.control(
                &approval,
                ApplicationWorkflowControlOutcome::Rejected,
                &rejected,
            );
        }
        Rejection::Repropose => {
            builder.retry(
                &approval,
                worth_query_host::facade::declaration::application_program::ApplicationWorkflowRetry::new(
                    ApplicationWorkflowControlOutcome::Rejected, "repropose", 2,
                ).expect("the reproposal bound is valid"),
                &propose,
            ).control(&approval, ApplicationWorkflowControlOutcome::RetryExhausted, &rejected);
        }
    }
    builder
        .finish()
        .expect("the authored definition is complete")
        .validate()
        .expect("the reviewed-document definition is valid")
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
