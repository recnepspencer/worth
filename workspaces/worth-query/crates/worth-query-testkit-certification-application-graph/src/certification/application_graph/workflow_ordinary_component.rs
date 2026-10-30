//! Public-host component authoring compiles into ordinary published progression.

pub use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorthQueryApplicationRequestExt,
    },
    declaration::application_program::{
        ApplicationWorkflowComponentBuilder, ApplicationWorkflowControlOutcome,
        ApplicationWorkflowDefinitionBuilder, ApplicationWorkflowEvidenceJoinPolicy,
        AuthoredWorkflowDefinition,
    },
};

pub use super::document_retention_model::{
    host::publish_workflow_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    retention_entry::{ReviewedSetRetentionBinding, DOCUMENT_IDENTITY},
    schema::DocumentRetentionQuery,
    workflow::{
        advance_instance, definition_limits, propose_instance, start_instance,
        ReviewedDocumentWorkflow, WorkflowApprovalCapability, WorkflowDefinitionAuthoringInput,
        WorkflowDefinitionAuthoringIntent, WorkflowDefinitionAuthoringOperation,
    },
};

pub fn reviewed_component_draft() -> AuthoredWorkflowDefinition<ReviewedDocumentWorkflow> {
    let mut review = ApplicationWorkflowComponentBuilder::<ReviewedDocumentWorkflow>::new(
        "required-document-review",
    )
    .expect("component identity is valid");
    let structural = review
        .assessment::<DocumentRetentionQuery>("structural")
        .expect("structural assessment is valid");
    let independent = review
        .assessment::<DocumentRetentionQuery>("independent")
        .expect("independent assessment is valid");
    let evidence = review
        .evidence_join(
            "evidence",
            ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        )
        .expect("join is valid");
    review
        .control(
            &structural,
            ApplicationWorkflowControlOutcome::Completed,
            &independent,
        )
        .expect("structural control is valid")
        .control(
            &independent,
            ApplicationWorkflowControlOutcome::Completed,
            &evidence,
        )
        .expect("independent control is valid")
        .assessment_evidence(&structural, &evidence)
        .expect("structural evidence is valid")
        .assessment_evidence(&independent, &evidence)
        .expect("independent evidence is valid");
    let structural_input = review
        .input_port("structural-subject", &structural)
        .expect("structural port is valid");
    let independent_input = review
        .input_port("independent-subject", &independent)
        .expect("independent port is valid");
    let evidence_output = review
        .output_port("reviewed-evidence", &evidence)
        .expect("evidence port is valid");
    let review = review.finish().expect("component closes");

    let mut workflow = ApplicationWorkflowDefinitionBuilder::<ReviewedDocumentWorkflow>::new(
        "ordinary-reviewed-document",
        definition_limits(),
    )
    .expect("workflow identity is valid");
    let proposal = workflow
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .expect("proposal is valid");
    let approval = workflow
        .approval::<WorkflowApprovalCapability>("approval")
        .expect("approval is valid");
    let apply = workflow
        .operation_binding::<ReviewedSetRetentionBinding>("apply")
        .expect("effect is valid");
    let completed = workflow.terminal("completed").expect("terminal is valid");
    let rejected = workflow.terminal("rejected").expect("terminal is valid");
    let checks = workflow
        .expand_component("checks", &review)
        .expect("component expands");
    let structural = checks
        .input(&structural_input)
        .expect("exported input binds");
    let independent = checks
        .input(&independent_input)
        .expect("exported input binds");
    let evidence = checks
        .output(&evidence_output)
        .expect("exported output binds");
    workflow
        .start(&proposal)
        .control(
            &proposal,
            ApplicationWorkflowControlOutcome::Completed,
            &structural,
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
            &approval,
            ApplicationWorkflowControlOutcome::Rejected,
            &rejected,
        )
        .control(
            &apply,
            ApplicationWorkflowControlOutcome::Completed,
            &completed,
        )
        .proposal_for_assessment(&proposal, &structural)
        .proposal_for_assessment(&proposal, &independent)
        .proposal_for_approval(&proposal, &approval)
        .joined_evidence(&evidence, &approval)
        .approval_authority(&approval, &apply)
        .operation_input(&proposal, &apply);
    workflow.finish().expect("reviewed workflow closes")
}
