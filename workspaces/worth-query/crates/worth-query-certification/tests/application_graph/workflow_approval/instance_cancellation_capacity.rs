//! A cancellation writes no transition, so an instance that has used its
//! whole retained-transition capacity still ends, warm or cold, and replays
//! exactly within the same bounds.

use worth_query_host::facade::application_entry::{
    WorkflowProposalPreparationDenial, WorthQueryWorkflowProposalPreparationDenial,
};
use worth_query_host::facade::declaration::application_program::ApplicationWorkflowComponentLimits;
use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;

use super::super::document_retention_model::{
    host::publish_on_first_program,
    workflow::{
        bounded_retry_definition_with_attempts, cancel_instance, propose_authoring_instance,
        retain_workflow_with_resources,
    },
};
use super::instance_cancellation::{cancellation_denial, cancelled};
use super::*;

const CAPACITY: u32 = 4;

/// An instance whose next proposal no longer fits its retained capacity.
fn at_capacity(key: u64) -> (DocumentWorkflowRuntime, PublishedWorkflowInstanceRef) {
    let resources = WorthQueryApplicationWorkflowResourceCeiling::new(
        32,
        64,
        4,
        ApplicationWorkflowComponentLimits::new(32, 4, 128, 256, 256).unwrap(),
        64 * 1024,
        32,
        CAPACITY,
        256 * 1024,
    )
    .expect("the workflow installation limits are nonzero");
    let application = retain_workflow_with_resources(publish_on_first_program(), resources);
    let definition = match publish_definition(
        &application,
        bounded_retry_definition_with_attempts(64),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key,
    )
    .expect("the retry definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the retry definition did not publish: {other:?}"),
    };
    let instance = match start_instance(&application, definition, key + 1)
        .expect("the instance start prepares")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("the instance did not start: {other:?}"),
    };
    for offset in 0..u64::from(CAPACITY) {
        assert!(matches!(
            propose_authoring_instance(&application, instance.clone(), key + 2 + offset),
            Ok(WorkflowProposalOutcome::Published(_))
        ));
    }
    match propose_authoring_instance(&application, instance.clone(), key + 20) {
        Err(WorthQueryWorkflowProposalPreparationDenial::ProposalPreparation(
            WorkflowProposalPreparationDenial::Attempt(denial),
        )) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable
        ),
        other => panic!("a full instance takes no further step: {other:?}"),
    }
    (application, instance)
}

#[test]
fn an_instance_that_used_its_whole_capacity_still_cancels_and_replays() {
    let (application, instance) = at_capacity(90_000);
    let ended = cancelled(cancel_instance(&application, instance.clone(), 90_030));
    assert!(!ended.replayed());
    assert!(ended.performed_node_paths().is_empty());
    let replay = cancelled(cancel_instance(&application, instance.clone(), 90_030));
    assert!(replay.replayed());
    assert_eq!(
        cancellation_denial(cancel_instance(&application, instance, 90_031)),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCancelled,
    );
}

#[test]
fn a_cold_full_instance_cancels_and_replays_from_settled_history() {
    let (application, instance) = at_capacity(90_100);
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    match propose_authoring_instance(&application, instance.clone(), 90_120) {
        Err(WorthQueryWorkflowProposalPreparationDenial::ProposalPreparation(
            WorkflowProposalPreparationDenial::Attempt(denial),
        )) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
            "a cold read refuses a full instance before commit, as a warm one does",
        ),
        other => panic!("a cold full instance takes no further step: {other:?}"),
    }
    let ended = cancelled(cancel_instance(&application, instance.clone(), 90_130));
    assert!(!ended.replayed());
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    let replay = cancelled(cancel_instance(&application, instance, 90_130));
    assert!(
        replay.replayed(),
        "the replay reads a full history in bounds"
    );
}
