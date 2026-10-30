//! Every live instance pins its definition revision, and one lineage holds at
//! most the installed number of pins. A full lineage refuses a new start with
//! a typed denial before any effect, while a retry of a recorded start still
//! replays. Publication takes no pin, a migration exchanges one pin for
//! another, and evicting compiled meaning never releases a pin.

pub use worth_query_host::facade::application_entry::WorthQueryWorkflowInstancePreparationDenial;
pub use worth_query_host::facade::declaration::application_program::ApplicationWorkflowComponentLimits;
pub use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;

pub use super::super::document_retention_model::{
    host::publish_on_first_program,
    workflow::{
        cancel_instance, continue_on_fork, expect_capacity_refused, expect_superseded_start,
        migrate_instance, retain_workflow_with_resources,
    },
};
pub use super::fork_continuation::fork_of;
pub use super::instance_cancellation::cancelled;
pub use super::instance_migration::{perform_approved_effect, replace_definition, started};
pub use super::journey::approval_requirement;
pub use super::*;

pub const PINS: u32 = 2;

pub type StartResult =
    Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstancePreparationDenial>;

/// A workflow host whose lineages each hold at most `PINS` live instances.
pub fn pinned(key: u64) -> (DocumentWorkflowRuntime, PublishedWorkflowDefinitionRef) {
    let resources = WorthQueryApplicationWorkflowResourceCeiling::new(
        32,
        64,
        4,
        ApplicationWorkflowComponentLimits::new(32, 4, 128, 256, 256).unwrap(),
        64 * 1024,
        PINS,
        64,
        256 * 1024,
    )
    .expect("the workflow installation limits are nonzero");
    let application = retain_workflow_with_resources(publish_on_first_program(), resources);
    let definition = match publish_definition(
        &application,
        reviewed_document_definition("completed"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key,
    )
    .expect("the reviewed definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the reviewed definition did not publish: {other:?}"),
    };
    (application, definition)
}

pub fn start(
    application: &DocumentWorkflowRuntime,
    definition: &PublishedWorkflowDefinitionRef,
    key: u64,
) -> StartResult {
    start_instance(application, definition.clone(), key)
}

/// The instance a start published, and whether it replayed.
pub fn admitted(outcome: StartResult) -> (PublishedWorkflowInstanceRef, bool) {
    let performed = started(outcome);
    (performed.instance().clone(), performed.replayed())
}

pub fn release_everything(application: &DocumentWorkflowRuntime) {
    let runtime = application.runtime();
    runtime.release_workflow_instance_progress_for_test();
    runtime.release_workflow_compilation_for_test();
}

pub fn assert_awaiting_assessment(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    key: u64,
) {
    match advance_instance(application, instance.clone(), key) {
        Ok(WorkflowProgressOutcome::AwaitingAssessment(_)) => {}
        other => panic!("the pinned instance must await its assessment: {other:?}"),
    }
}
