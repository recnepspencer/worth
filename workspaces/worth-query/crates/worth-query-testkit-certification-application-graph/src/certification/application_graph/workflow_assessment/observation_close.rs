//! An assessment observation holds interest in its run and nothing more.
//! Closing or dropping one releases only that observer's interest: another
//! observer's run keeps going, and the instance keeps awaiting its assessment
//! until a later observer settles it. Wakes coalesce into the run's single
//! generation, so an observer never has more than one pending notification.

pub use super::super::document_retention_model::{
    assessment_output::RetentionAssessmentDemand,
    host::DocumentWorkflowRuntime,
    operator_identity::{authenticate_operator, request_scope},
    schema::{DocumentRetentionQuery, DocumentRetentionSchema},
    workflow::{
        cancel_instance, ReviewedDocumentWorkflow, WorkflowAdvanceInput, WorkflowAdvanceIntent,
    },
};
pub use super::*;
pub use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, WorkflowInstanceCancellationOutcome,
    WorthQueryApplicationRequestExt, WorthQueryOutputDemandControls,
    WorthQueryWorkflowAssessmentDemandHandle, WorthQueryWorkflowAssessmentDemandProgress,
    WorthQueryWorkflowAssessmentDemandSettlement,
};

/// A published instance awaiting its assessment head.
pub fn awaiting_assessment(key: u64) -> (DocumentWorkflowRuntime, PublishedWorkflowInstanceRef) {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        reviewed_document_definition("completed"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key,
    )
    .expect("the definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(published) => {
            published.definition().clone()
        }
        other => panic!("the definition did not publish: {other:?}"),
    };
    let instance = match start_instance(&application, definition, key + 1)
        .expect("the instance start prepares")
    {
        WorkflowInstanceStartOutcome::Started(started) => started.instance().clone(),
        other => panic!("the instance did not start: {other:?}"),
    };
    propose_instance(&application, instance.clone(), key + 2).expect("the instance proposes");
    assert_awaiting(&application, &instance, key + 3);
    (application, instance)
}

pub fn assert_awaiting(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    key: u64,
) {
    match advance_instance(application, instance.clone(), key)
        .expect("the assessment head prepares")
    {
        WorkflowProgressOutcome::AwaitingAssessment(_) => {}
        other => panic!("the instance must still await its assessment: {other:?}"),
    }
}

pub type Observer<'application> = WorthQueryWorkflowAssessmentDemandHandle<
    'application,
    DocumentRetentionSchema,
    ReviewedDocumentWorkflow,
    RetentionAssessmentDemand,
>;

pub fn observe<'application>(
    application: &'application DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    key: u64,
    attempts: usize,
) -> Observer<'application> {
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
        .idempotency(&key)
        .prepare_workflow_advance(application, instance.clone())
        .expect("the assessment head admits an observer")
        .into_assessment_demand(RetentionAssessmentDemand::new(DOCUMENT_IDENTITY))
        .expect("the demand matches the installed assessment contract")
        .controls(
            WorthQueryOutputDemandControls::default()
                .settlement_attempts(std::num::NonZeroUsize::new(attempts).unwrap()),
        )
        .advanced()
        .start()
        .expect("the assessment observation starts")
}

pub fn progress(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    observer: &mut Observer<'_>,
) -> Result<
    WorthQueryWorkflowAssessmentDemandProgress<DocumentRetentionQuery>,
    WorthQueryApplicationOutputDemandDenial,
> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    observer.advanced().settle(
        &runtime
            .request(&principal, &scope)
            .on_branch(instance.branch()),
    )
}

pub fn settle(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    observer: &mut Observer<'_>,
) -> Result<
    WorthQueryWorkflowAssessmentDemandSettlement<DocumentRetentionQuery>,
    WorthQueryApplicationOutputDemandDenial,
> {
    loop {
        match progress(application, instance, observer)? {
            WorthQueryWorkflowAssessmentDemandProgress::Settled(settled) => return Ok(settled),
            WorthQueryWorkflowAssessmentDemandProgress::Progressed => continue,
            WorthQueryWorkflowAssessmentDemandProgress::Pending => {
                panic!("the synchronous assessment is waiting for external progress")
            }
        }
    }
}

/// Advances the observer into a run whose readiness delivery has not arrived:
/// the one wait a call cannot finish by itself.
pub fn in_flight(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    observer: &mut Observer<'_>,
) {
    loop {
        application
            .runtime()
            .delay_next_output_readiness_delivery_for_test();
        match progress(application, instance, observer)
            .expect("the original observer advances to its deferred native delivery")
        {
            WorthQueryWorkflowAssessmentDemandProgress::Pending => return,
            WorthQueryWorkflowAssessmentDemandProgress::Progressed => continue,
            WorthQueryWorkflowAssessmentDemandProgress::Settled(_) => {
                panic!("an undelivered readiness must leave the run in flight")
            }
        }
    }
}

pub fn closed<Value>(outcome: Result<Value, WorthQueryApplicationOutputDemandDenial>) {
    match outcome.err() {
        Some(WorthQueryApplicationOutputDemandDenial::Demand(denial))
            if denial.kind() == WorthQueryOutputDemandDenialKind::Closed => {}
        other => panic!("a closed observation must refuse as closed: {other:?}"),
    }
}

pub fn accepted_past_assessment(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    settled: &WorthQueryWorkflowAssessmentDemandSettlement<DocumentRetentionQuery>,
    key: u64,
) {
    match accept_assessment(application, instance.clone(), settled, key)
        .expect("the settled assessment is accepted")
    {
        WorkflowProgressOutcome::AwaitingAssessment(_) => {
            panic!("an accepted assessment must move the instance past its head")
        }
        WorkflowProgressOutcome::PreparationDenied(denial) => {
            panic!("the accepted assessment was refused: {denial:?}")
        }
        _ => {}
    }
}
