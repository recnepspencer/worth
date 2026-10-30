//! Definition readmission through native checkpoint restore and cold compilation.

pub use super::document_retention_model::{
    host::{publish_workflow_on_first_program, restore_on_first_program, DocumentWorkflowRuntime},
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
    application_installation::{
        WorthQueryApplicationCheckpoint, WorthQueryApplicationProgramRoster,
        WorthQueryCheckpointCapturePolicy,
    },
};

pub fn capture(
    application: &DocumentWorkflowRuntime<
        impl worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition<
            DocumentRetentionSchema,
        >,
        impl worth_query_host::facade::declaration::application_program::ApplicationWorkflowSpec<
            Schema = DocumentRetentionSchema,
        >,
    >,
) -> Vec<u8> {
    application
        .with_host_advancement(|phase| {
            application
                .runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
                .bytes()
                .to_vec()
        })
        .unwrap()
}

pub fn restore(
    bytes: Vec<u8>,
) -> DocumentWorkflowRuntime<
    WorkflowOwnerProgram,
    worth_query_testkit_document_retention::facade::document_retention_model::workflow::governed_read::GovernedDocumentWorkflow,
>{
    let program =
        worth_query_host::facade::declaration::application_program::ApplicationProgramAuthoring::<
            DocumentRetentionSchema,
            WorkflowOwnerProgram,
        >::begin()
        .validated_program()
        .unwrap();
    worth_query_testkit_document_retention::facade::document_retention_model::workflow::retain_workflow_for(
        super::document_retention_model::host::restore(
            program,
            WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes),
            WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        )
        .expect("the original program and native checkpoint restore before definition readmission"),
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
