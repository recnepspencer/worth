use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryOutputDemandControls,
    WorthQueryWorkflowAssessmentDemandProgress, WorthQueryWorkflowAssessmentDemandSettlement,
};

use super::super::{
    assessment_output::{LookalikePartAssessmentDemand, PartAssessmentDemand},
    dimension_entry::PART_IDENTITY,
    host::BoundedDimensionWorkflowRuntime,
    operator_identity::{authenticate_operator, request_scope},
    schema::PartDimensionQuery,
};
use super::{WorkflowAdvanceInput, WorkflowAdvanceIntent};

pub fn accept_assessment(
    application: &BoundedDimensionWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    settlement: &WorthQueryWorkflowAssessmentDemandSettlement<PartDimensionQuery>,
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
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_advance(application, instance)
        .expect("assessment acceptance must receive fresh workflow admission")
        .accept_assessment(settlement)
}

pub fn settle_assessment(
    application: &BoundedDimensionWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> WorthQueryWorkflowAssessmentDemandSettlement<PartDimensionQuery> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let mut handle = runtime
        .request(&principal, &scope)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_advance(application, instance)
        .expect("assessment-head workflow admission must succeed")
        .into_assessment_demand(PartAssessmentDemand::new(PART_IDENTITY))
        .expect("the typed demand must match the installed assessment contract")
        .controls(WorthQueryOutputDemandControls::new(
            std::num::NonZeroUsize::new(512).unwrap(),
            std::num::NonZeroUsize::new(1024).unwrap(),
        ))
        .start()
        .expect("the installed assessment output demand must start");
    for _ in 0..4 {
        match handle
            .settle(&runtime.request(&principal, &scope))
            .expect("the installed assessment producer must advance")
        {
            WorthQueryWorkflowAssessmentDemandProgress::Pending => {}
            WorthQueryWorkflowAssessmentDemandProgress::Settled(settled) => return settled,
        }
    }
    panic!("the bounded assessment producer did not settle within its declared work")
}

pub fn spoofed_assessment_denial(
    application: &BoundedDimensionWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime
        .request(&principal, &scope)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_advance(application, instance)
        .expect("spoof probe must receive fresh workflow admission")
        .into_assessment_demand(LookalikePartAssessmentDemand::new(PART_IDENTITY))
        .expect("the lookalike repeats the public string contract");
    match request.start() {
        Ok(mut handle) => {
            handle.close();
            panic!("lookalike family type must not select the installed producer")
        }
        Err(denial) => denial,
    }
}
