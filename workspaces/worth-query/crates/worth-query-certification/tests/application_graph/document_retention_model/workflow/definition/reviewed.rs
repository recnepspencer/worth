use super::*;

pub(super) enum Rejection {
    Terminal,
    Retry(u16),
    Repropose,
}

pub(super) fn reviewed_document_definition_with_policy<
    Spec: worth_query_host::facade::declaration::application_program::ApplicationWorkflowSpec<
        Schema = DocumentRetentionSchema,
    >,
    Binding: worth_query_host::facade::declaration::application_operation::ApplicationMutationBinding<
        DocumentRetentionSchema,
    >,
>(
    completion_identity: &str,
    join_policy: worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy,
    rejection: Rejection,
    limits: ApplicationWorkflowDefinitionLimits,
) -> ValidatedWorkflowDefinition<Spec> {
    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<Spec>::new("reviewed-document", limits)
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
        .operation_binding::<Binding>("apply")
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
