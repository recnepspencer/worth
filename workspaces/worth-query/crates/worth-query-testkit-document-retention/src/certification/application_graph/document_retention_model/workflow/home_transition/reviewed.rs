//! The reviewed document graph whose proposal captures the guarded
//! operation's target input.
use super::super::super::schema::DocumentRetentionQuery;
use super::super::governed_read::GovernedDocumentWorkflow;
use super::super::{
    definition_limits, WorkflowApprovalCapability, WorkflowDefinitionAuthoringBinding,
};
use super::TargetNode;
use worth_query_decl::facade::application_program::*;

pub fn reviewed() -> (
    ValidatedWorkflowDefinition<GovernedDocumentWorkflow>,
    TargetNode,
) {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<GovernedDocumentWorkflow>::new(
        "reviewed-document",
        definition_limits(),
    )
    .expect("the workflow identity is valid");
    let propose = builder
        .operation_binding::<WorkflowDefinitionAuthoringBinding>("propose")
        .expect("the proposal node is valid");
    let structural = builder
        .assessment::<DocumentRetentionQuery>("checks/structural")
        .expect("the structural assessment node is valid");
    let compliance = builder
        .assessment::<DocumentRetentionQuery>("checks/compliance")
        .expect("the compliance assessment node is valid");
    let evidence = builder
        .evidence_join(
            "checks/evidence",
            ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        )
        .expect("the evidence node is valid");
    let approval = builder
        .approval::<WorkflowApprovalCapability>("approval")
        .expect("the approval node is valid");
    let apply = builder
        .operation_binding::<super::super::super::retention_entry::ReviewedSetRetentionBinding>(
            "apply",
        )
        .expect("the guarded operation node is valid");
    let completed = builder
        .terminal("approved")
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
        .captured_operation_input(&propose, &apply);
    builder.control(
        &approval,
        ApplicationWorkflowControlOutcome::Rejected,
        &rejected,
    );
    (builder.finish().unwrap().validate().unwrap(), apply)
}
