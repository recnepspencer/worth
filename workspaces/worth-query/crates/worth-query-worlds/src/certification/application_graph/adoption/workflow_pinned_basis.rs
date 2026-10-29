//! A live instance decides over the definition revision it started under. A
//! new revision or a new lineage capacity changes what a new start reads, and
//! leaves a waiting instance nothing to reconsider.

pub use worth_query_host::facade::application_entry::{
    PublishedWorkflowDefinitionRef, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowProgressOutcome,
};
pub use worth_query_host::facade::declaration::application_program::ApplicationWorkflowComponentLimits;
pub use worth_query_host::facade::domain::WorthQueryApplicationWorkflowResourceCeiling;

pub use super::workflow_participant::expect_started;
pub use crate::facade::certification::document_retention_model::{
    host::{publish_on_first_program, DocumentWorkflowRuntime},
    programs::RetentionProgramP1,
    workflow::{
        advance_instance, expect_capacity_refused, expect_superseded_start, install_workflow_spec,
        prepare_second_program_adoption, publish_adoption, publish_definition,
        retain_workflow_with_resources, start_instance, terminal_definition,
    },
};

/// The installed ceiling with `live_instances` live instances per lineage.
pub fn ceiling(live_instances: u32) -> WorthQueryApplicationWorkflowResourceCeiling {
    WorthQueryApplicationWorkflowResourceCeiling::new(
        32,
        64,
        4,
        ApplicationWorkflowComponentLimits::new(32, 4, 128, 256, 256).unwrap(),
        64 * 1024,
        live_instances,
        128,
        256 * 1024,
    )
    .expect("the workflow installation limits are nonzero")
}

pub fn published(
    application: &DocumentWorkflowRuntime,
    terminal: &str,
    predecessor: WorkflowDefinitionExpectedPredecessor,
    key: u64,
) -> PublishedWorkflowDefinitionRef {
    match publish_definition(application, terminal_definition(terminal), predecessor, key)
        .expect("the definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the {terminal} definition did not publish: {other:?}"),
    }
}
