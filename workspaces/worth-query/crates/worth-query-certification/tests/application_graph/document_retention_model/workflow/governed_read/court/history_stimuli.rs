//! Existing production stimuli with this isolated governed Spec; no read-derived authority.
use super::*;
use crate::document_retention_model::{
    assessment_output::RetentionAssessmentDemand,
    operator_identity::block_on,
    schema::DocumentRetentionQuery,
    workflow::{
        WorkflowAdvanceInput, WorkflowAdvanceIntent, WorkflowApprovalIntent,
        WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringIntent,
    },
};
pub fn advance_instance_on(
    application: &GovernedRuntime,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowAdvancePreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_advance(application, instance)
        .map(|request| request.execute())
}

pub fn approve_instance(
    application: &GovernedRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    authentication: &crate::document_retention_model::workflow::CertificationAuthenticationOwner,
    required: &RequiredWorkflowApproval,
    proposal: &PublishedWorkflowProposalRef,
    decision: WorkflowApprovalDecision,
    idempotency: u64,
) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowAdvancePreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = crate::document_retention_model::operator_identity::authenticate_reviewer(
        runtime.installed_schema(),
        &scope,
    );
    let signing = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowApprovalIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_approval(application, instance, required, proposal, decision)?;
    let Some(authentication_intent) = signing.authentication_intent().cloned() else {
        return signing.execute_replay();
    };
    let event =
        block_on(authentication.authenticate((), &principal, authentication_intent, &scope))
            .expect("the installed certification factor accepts the exact approval challenge");
    signing.sign(&event).map(|request| request.execute())
}

pub fn propose_authoring_instance_with_retention(
    application: &GovernedRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
    retention_days: u64,
) -> Result<WorkflowProposalOutcome, WorthQueryWorkflowProposalPreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days,
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_proposal(application, instance)
        .map(|request| request.execute())
}

pub fn accept_assessment(
    application: &GovernedRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    settlement: &WorthQueryWorkflowAssessmentDemandSettlement<DocumentRetentionQuery>,
    idempotency: u64,
) -> Result<
    worth_query_host::facade::application_entry::WorkflowProgressOutcome,
    worth_query_host::facade::application_entry::WorthQueryWorkflowAssessmentAcceptanceDenial,
> {
    let runtime = application.runtime();
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
        .prepare_workflow_advance(application, instance)
        .expect("assessment acceptance must receive fresh workflow admission")
        .accept_assessment(settlement)
}

pub fn settle_assessment_for(
    application: &GovernedRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
    subject_identity: &str,
) -> WorthQueryWorkflowAssessmentDemandSettlement<DocumentRetentionQuery> {
    let runtime = application.runtime();
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
        .prepare_workflow_advance(application, instance)
        .expect("assessment-head workflow admission must succeed")
        .into_assessment_demand(RetentionAssessmentDemand::new(subject_identity))
        .expect("the typed demand must match the installed assessment contract")
        .retain_operation(application.program_runtime())
        .expect("the installed assessment output demand must start");
    drop(principal);
    drop(scope);
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(runtime.installed_schema(), &fresh_scope);
    let request = runtime
        .request(&fresh_principal, &fresh_scope)
        .on_branch(branch);
    let settlement = match handle
        .advance(&request)
        .expect("the installed assessment producer must advance")
    {
        WorthQueryWorkflowAssessmentDemandProgress::Settled(settled) => settled,
        WorthQueryWorkflowAssessmentDemandProgress::Pending => {
            panic!("the bounded assessment producer did not settle within its declared work")
        }
    };
    handle.close(&request).unwrap_or_else(|_| {
        panic!("fresh native observer close retains failed cleanup on refusal")
    });
    settlement
}
