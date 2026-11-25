//! Definition readmission through Directory Home reopen and cold compilation.

pub use super::document_retention_model::{
    host::{
        host_limits, open_workflow_directory_for, publish_workflow_on_first_program,
        DocumentWorkflowApplication, DocumentWorkflowDirectoryHome,
    },
    programs::{validated_second_program, OccurrenceAction, WorkflowOwnerProgram},
    retention_entry::OccurrenceSetRetentionBinding,
    schema::DocumentRetentionSchema,
    workflow::{advance_instance, propose_authoring_instance_with_retention, publish_definition},
};
pub use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
    },
    application_installation::WorthQueryApplicationProgramRoster,
};
pub use worth_query_testkit_document_retention::facade::directory_home::unique_directory;

pub fn open_owner_workflow_directory(
    directory: std::path::PathBuf,
) -> DocumentWorkflowDirectoryHome<
    WorkflowOwnerProgram,
    worth_query_testkit_document_retention::facade::document_retention_model::workflow::governed_read::GovernedDocumentWorkflow,
> {
    let program =
        worth_query_host::facade::declaration::application_program::ApplicationProgramAuthoring::<
            DocumentRetentionSchema,
            WorkflowOwnerProgram,
        >::begin()
        .validated_program()
        .unwrap();
    open_workflow_directory_for(
        directory,
        program,
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        worth_query_testkit_document_retention::facade::document_retention_model::workflow::workflow_resources(),
    )
}

#[path = "workflow_definition_readmission/current.rs"]
pub mod current;

// WQWD's admitted current header: spec, observation policy, actor policy.
// This creates hostile current metadata, never a purported old export.
pub fn replace_header_text(bytes: &[u8], field: usize, replacement: &str) -> Vec<u8> {
    let mut position = 6;
    for _ in 0..field {
        let length = u32::from_be_bytes(bytes[position..position + 4].try_into().unwrap()) as usize;
        position += 4 + length;
    }
    let length = u32::from_be_bytes(bytes[position..position + 4].try_into().unwrap()) as usize;
    let mut altered = bytes[..position].to_vec();
    altered.extend_from_slice(&(replacement.len() as u32).to_be_bytes());
    altered.extend_from_slice(replacement.as_bytes());
    altered.extend_from_slice(&bytes[position + 4 + length..]);
    altered
}
