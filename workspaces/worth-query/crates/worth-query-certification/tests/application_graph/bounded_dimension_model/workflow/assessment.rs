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

pub fn accept_early_assessment(
    application: &BoundedDimensionWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    node_path: &str,
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
        .prepare_workflow_collect_assessment(application, instance, node_path)
        .expect("early assessment acceptance must receive fresh workflow admission")
        .accept_assessment(settlement)
}

pub fn settle_early_assessment_for(
    application: &BoundedDimensionWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    node_path: &str,
    idempotency: u64,
    subject_identity: &str,
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
        .prepare_workflow_collect_assessment(application, instance, node_path)
        .expect("declared early assessment must be ready")
        .into_assessment_demand(PartAssessmentDemand::new(subject_identity))
        .expect("the typed demand must match the installed assessment contract")
        .controls(WorthQueryOutputDemandControls::new(
            std::num::NonZeroUsize::new(512).unwrap(),
            std::num::NonZeroUsize::new(1024).unwrap(),
        ))
        .start()
        .expect("the installed early assessment demand must start");
    match handle
        .settle(&runtime.request(&principal, &scope))
        .expect("the installed assessment producer must advance")
    {
        WorthQueryWorkflowAssessmentDemandProgress::Settled(settled) => settled,
        WorthQueryWorkflowAssessmentDemandProgress::Pending => {
            panic!("the bounded assessment producer did not settle within its declared work")
        }
    }
}

pub fn prepare_early_assessment_denial(
    application: &BoundedDimensionWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    node_path: &str,
    idempotency: u64,
) -> worth_query_host::facade::application_entry::WorthQueryWorkflowAdvancePreparationDenial {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    match runtime
        .request(&principal, &scope)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_collect_assessment(application, instance, node_path)
    {
        Ok(_) => panic!("unready or non-assessment node must not collect evidence"),
        Err(denial) => denial,
    }
}

pub fn settle_assessment(
    application: &BoundedDimensionWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> WorthQueryWorkflowAssessmentDemandSettlement<PartDimensionQuery> {
    settle_assessment_for(application, instance, idempotency, PART_IDENTITY)
}

pub fn settle_assessment_for(
    application: &BoundedDimensionWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
    subject_identity: &str,
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
        .into_assessment_demand(PartAssessmentDemand::new(subject_identity))
        .expect("the typed demand must match the installed assessment contract")
        .controls(WorthQueryOutputDemandControls::new(
            std::num::NonZeroUsize::new(512).unwrap(),
            std::num::NonZeroUsize::new(1024).unwrap(),
        ))
        .start()
        .expect("the installed assessment output demand must start");
    match handle
        .settle(&runtime.request(&principal, &scope))
        .expect("the installed assessment producer must advance")
    {
        WorthQueryWorkflowAssessmentDemandProgress::Settled(settled) => settled,
        WorthQueryWorkflowAssessmentDemandProgress::Pending => {
            panic!("the bounded assessment producer did not settle within its declared work")
        }
    }
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
