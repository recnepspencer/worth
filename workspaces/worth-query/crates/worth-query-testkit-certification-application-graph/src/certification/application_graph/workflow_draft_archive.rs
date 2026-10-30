//! Public certification that an authored definition travels as a bounded,
//! authority-free draft and authors again only against installed vocabulary.

pub use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    },
    declaration::application_program::{ApplicationWorkflowNodeKind, ValidatedWorkflowDefinition},
};
pub use worth_query_package_archive::facade::{
    decode_workflow_definition_draft, encode_workflow_definition_draft,
    WorthQueryPackageArchiveLimits, WorthQueryUntrustedWorkflowDefinitionDraft,
    WorthQueryWorkflowDefinitionDraftDenialKind as DraftDenial,
};

pub use super::document_retention_model::{
    host::{publish_workflow_on_first_program, DocumentWorkflowRuntime},
    workflow::{
        approval_retry_definition, bounded_retry_definition, condition_terminal_definition,
        conditionally_required_related_assessment_definition, publish_definition,
        reviewed_document_definition, start_instance, ReviewedDocumentWorkflow,
    },
};
pub use super::document_retention_model::{
    retention_entry::{ExplicitReviewedSetRetentionBinding, OccurrenceSetRetentionBinding},
    workflow::reviewed_document_definition_for_binding,
};
pub use super::workflow_component_scale::component_definition;

pub type Definition = ValidatedWorkflowDefinition<ReviewedDocumentWorkflow>;

pub fn encode(definition: &Definition) -> Vec<u8> {
    encode_workflow_definition_draft(definition, WorthQueryPackageArchiveLimits::DEFAULT)
        .expect("the definition fits the archive budget")
}

pub fn decode(bytes: &[u8]) -> WorthQueryUntrustedWorkflowDefinitionDraft {
    decode_workflow_definition_draft(bytes, WorthQueryPackageArchiveLimits::DEFAULT)
        .expect("the draft bytes are well formed")
}

pub fn round_trip(application: &DocumentWorkflowRuntime, bytes: &[u8]) -> Definition {
    decode(bytes)
        .author(application.workflow_spec())
        .expect("the draft authors against installed vocabulary")
        .validate()
        .expect("the authored draft is valid")
}

pub fn refusal(application: &DocumentWorkflowRuntime, bytes: &[u8]) -> DraftDenial {
    match decode(bytes).author(application.workflow_spec()) {
        Ok(_) => panic!("the draft must not author against installed vocabulary"),
        Err(denial) => denial.kind(),
    }
}

/// Replaces only the header's start, which precedes every node record.
pub fn restart(bytes: &[u8], start: &str, replacement: &str) -> Vec<u8> {
    let marker = framed(start);
    let position = bytes
        .windows(marker.len())
        .position(|window| window == marker.as_slice())
        .expect("the header names the start");
    let mut output = bytes[..position].to_vec();
    output.extend_from_slice(&framed(replacement));
    output.extend_from_slice(&bytes[position + marker.len()..]);
    output
}

pub fn framed(text: &str) -> Vec<u8> {
    let length = u32::try_from(text.len()).expect("test text is short");
    let mut bytes = length.to_be_bytes().to_vec();
    bytes.extend_from_slice(text.as_bytes());
    bytes
}

/// Replaces every length-framed occurrence of `from` with `to`.
pub fn retext(bytes: &[u8], from: &str, to: &str) -> Vec<u8> {
    replace(bytes, &framed(from), &framed(to))
}

/// Replaces every occurrence of the byte sequence `from` with `to`.
pub fn replace(bytes: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    let mut replaced = false;
    while index < bytes.len() {
        if bytes[index..].starts_with(from) {
            output.extend_from_slice(to);
            index += from.len();
            replaced = true;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    assert!(replaced, "the draft carries the text being replaced");
    output
}
