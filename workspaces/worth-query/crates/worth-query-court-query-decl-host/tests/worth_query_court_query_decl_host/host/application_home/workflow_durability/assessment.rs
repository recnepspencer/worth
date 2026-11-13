//! Assessment observers settle and close before any Home close.
use super::*;
use document::{assessment_output::RetentionAssessmentDemand, schema::DocumentRetentionQuery};
pub(super) fn settle(
    application: &Home,
    instance: PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> WorthQueryWorkflowAssessmentDemandSettlement<DocumentRetentionQuery> {
    let runtime = application.runtime().runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let branch = instance.branch();
    let mut handle = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_advance(application.vocabulary(), instance)
        .expect("assessment-head workflow admission must succeed")
        .into_assessment_demand(RetentionAssessmentDemand::new(DOCUMENT_IDENTITY))
        .expect("the typed demand must match the installed assessment contract")
        .retain_operation(application.runtime())
        .expect("the installed assessment output demand must start");
    drop(principal);
    drop(scope);
    loop {
        let fresh_scope = request_scope();
        let fresh_principal = authenticate_operator(runtime.installed_schema(), &fresh_scope);
        let request = runtime
            .request(&fresh_principal, &fresh_scope)
            .on_branch(branch);
        match handle
            .advance(&request)
            .expect("the installed assessment producer must advance")
        {
            WorthQueryWorkflowAssessmentDemandProgress::Settled(settled) => {
                handle.close(&request).unwrap_or_else(|_| {
                    panic!("fresh native observer close retains failed cleanup on refusal")
                });
                return settled;
            }
            WorthQueryWorkflowAssessmentDemandProgress::Progressed => continue,
            WorthQueryWorkflowAssessmentDemandProgress::Pending => {
                panic!("the synchronous assessment is waiting for external progress")
            }
        }
    }
}
pub(super) fn accept(
    application: &Home,
    instance: PublishedWorkflowInstanceRef,
    settlement: &WorthQueryWorkflowAssessmentDemandSettlement<DocumentRetentionQuery>,
    idempotency: u64,
) -> WorkflowProgressOutcome {
    let runtime = application.runtime().runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_advance(application.vocabulary(), instance)
        .expect("assessment acceptance must receive fresh workflow admission")
        .accept_assessment(settlement)
        .unwrap()
}
