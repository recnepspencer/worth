//! A cancellation writes no transition, so an instance that has used its
//! whole retained-transition capacity still ends, warm or cold, and replays
//! exactly within the same bounds.

pub use worth_query_host::facade::application_entry::{
    WorkflowProposalPreparationDenial, WorthQueryWorkflowProposalPreparationDenial,
};
pub use worth_query_host::facade::declaration::application_program::ApplicationWorkflowComponentLimits;
pub use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;

pub use super::super::document_retention_model::{
    host::publish_on_first_program,
    workflow::{
        bounded_retry_definition_with_attempts, cancel_instance, propose_authoring_instance,
        retain_workflow_with_resources,
    },
};
pub use super::instance_cancellation::{cancellation_denial, cancelled};
pub use super::*;

pub const CAPACITY: u32 = 4;

/// An instance whose next proposal no longer fits its retained capacity.
pub fn at_capacity(key: u64) -> (DocumentWorkflowRuntime, PublishedWorkflowInstanceRef) {
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
