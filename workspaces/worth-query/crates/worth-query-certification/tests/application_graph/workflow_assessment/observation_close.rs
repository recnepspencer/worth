//! An assessment observation holds interest in its run and nothing more.
//! Closing or dropping one releases only that observer's interest: another
//! observer's run keeps going, and the instance keeps awaiting its assessment
//! until a later observer settles it. Wakes coalesce into the run's single
//! generation, so an observer never has more than one pending notification.

use super::super::document_retention_model::{
    assessment_output::RetentionAssessmentDemand,
    host::DocumentWorkflowRuntime,
    operator_identity::{authenticate_operator, request_scope},
    schema::{DocumentRetentionQuery, DocumentRetentionSchema},
    workflow::{
        cancel_instance, ReviewedDocumentWorkflow, WorkflowAdvanceInput, WorkflowAdvanceIntent,
    },
};
use super::*;
use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, WorkflowInstanceCancellationOutcome,
    WorthQueryApplicationRequestExt, WorthQueryOutputDemandControls,
    WorthQueryWorkflowAssessmentDemandHandle, WorthQueryWorkflowAssessmentDemandProgress,
    WorthQueryWorkflowAssessmentDemandSettlement,
};

/// A published instance awaiting its assessment head.
fn awaiting_assessment(key: u64) -> (DocumentWorkflowRuntime, PublishedWorkflowInstanceRef) {
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

fn assert_awaiting(
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

type Observer<'application> = WorthQueryWorkflowAssessmentDemandHandle<
    'application,
    DocumentRetentionSchema,
    ReviewedDocumentWorkflow,
    RetentionAssessmentDemand,
>;

fn observe<'application>(
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
            WorthQueryOutputDemandControls::new(
                std::num::NonZeroUsize::new(512).unwrap(),
                std::num::NonZeroUsize::new(1024).unwrap(),
            )
            .settlement_attempts(std::num::NonZeroUsize::new(attempts).unwrap()),
        )
        .start()
        .expect("the assessment observation starts")
}

fn progress(
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
    observer.settle(
        &runtime
            .request(&principal, &scope)
            .on_branch(instance.branch()),
    )
}

fn settle(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    observer: &mut Observer<'_>,
) -> Result<
    WorthQueryWorkflowAssessmentDemandSettlement<DocumentRetentionQuery>,
    WorthQueryApplicationOutputDemandDenial,
> {
    match progress(application, instance, observer)? {
        WorthQueryWorkflowAssessmentDemandProgress::Settled(settled) => Ok(settled),
        WorthQueryWorkflowAssessmentDemandProgress::Pending => {
            panic!("the bounded assessment producer did not settle within its declared work")
        }
    }
}

fn closed<Value>(outcome: Result<Value, WorthQueryApplicationOutputDemandDenial>) {
    match outcome.err() {
        Some(WorthQueryApplicationOutputDemandDenial::Demand(denial))
            if denial.kind() == WorthQueryOutputDemandDenialKind::Closed => {}
        other => panic!("a closed observation must refuse as closed: {other:?}"),
    }
}

fn accepted_past_assessment(
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

#[test]
fn closing_one_observer_leaves_the_other_settling_the_shared_run() {
    let (application, instance) = awaiting_assessment(94_000);
    let mut first = observe(&application, &instance, 94_010, 1);
    assert!(
        matches!(
            progress(&application, &instance, &mut first),
            Ok(WorthQueryWorkflowAssessmentDemandProgress::Pending)
        ),
        "one settlement attempt leaves the run in flight",
    );
    let mut second = observe(&application, &instance, 94_011, 64);
    let joined = second
        .notifications()
        .expect("an open observation has notifications")
        .generation();
    assert!(
        joined > 0,
        "the second observer joins the run already in flight"
    );
    first.close();
    closed(first.notifications());
    closed(progress(&application, &instance, &mut first));

    let settled = settle(&application, &instance, &mut second)
        .expect("the remaining observer settles the run the closed one started");
    accepted_past_assessment(&application, &instance, &settled, 94_012);
}

#[test]
fn closing_every_observer_leaves_the_instance_awaiting_its_assessment() {
    let (application, instance) = awaiting_assessment(94_100);
    let mut closed_observer = observe(&application, &instance, 94_110, 1);
    assert!(
        matches!(
            progress(&application, &instance, &mut closed_observer),
            Ok(WorthQueryWorkflowAssessmentDemandProgress::Pending)
        ),
        "the closed observer releases a run it had in flight",
    );
    closed_observer.close();
    drop(observe(&application, &instance, 94_111, 64));
    drop(closed_observer);
    assert_awaiting(&application, &instance, 94_112);

    let mut later = observe(&application, &instance, 94_113, 64);
    let settled = settle(&application, &instance, &mut later)
        .expect("a later observer settles the released assessment");
    accepted_past_assessment(&application, &instance, &settled, 94_114);
    drop(later);
    match cancel_instance(&application, instance, 94_115).expect("the cancellation prepares") {
        WorkflowInstanceCancellationOutcome::Cancelled(ended) => {
            assert!(
                !ended.replayed(),
                "closing observers did not end the instance"
            )
        }
        other => panic!("the instance must still cancel: {other:?}"),
    }
}

#[test]
fn an_observer_holds_at_most_one_pending_wake_for_its_run() {
    let (application, instance) = awaiting_assessment(94_200);
    let watcher = observe(&application, &instance, 94_210, 64);
    let mut settler = observe(&application, &instance, 94_211, 64);
    let notifications = watcher
        .notifications()
        .expect("an open observation has notifications");
    let before = notifications.generation();

    settle(&application, &instance, &mut settler).expect("the shared run settles");
    let after = notifications.generation();
    assert!(
        after > before + 1,
        "the shared run changed state more than once: {before} -> {after}",
    );
    assert_eq!(
        notifications.wait_after(before),
        after,
        "every change since the observer last looked is one pending wake",
    );
}
