//! Canonical operation identity through real bindings and the ordinary
//! application request API.
//!
//! An author never writes an input identity: Query derives it from every field
//! of the serialized input. These proofs show what that buys a caller: text
//! that only moves a delimiter between fields is a different request, and any
//! changed field under a reused idempotency key is intent drift, never a replay.
//! A request encodes its input and its key once each, and its receipt reports
//! exactly those two derivations in admission, whether it commits or replays.

pub use worth_query_decl::facade::application_operation::ApplicationMutationIdentities;
pub use worth_query_host::facade::domain::WorthQueryCanonicalWorkEvidence;

pub use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

pub use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
};

pub use crate::facade::certification::document_retention_model::host::{
    publish_on_first_program, publish_workflow_on_first_program, SEED_RETENTION,
};
pub use crate::facade::certification::document_retention_model::workflow::{
    publish_definition, reviewed_document_definition,
};

pub use crate::facade::certification::document_retention_model::presented_request::set_retention;
pub use crate::facade::certification::document_retention_model::readback::read_retention;
pub use crate::facade::certification::document_retention_model::schema::{
    DocumentRetentionSchema, SetRetentionInput,
};
pub use crate::facade::certification::document_retention_model::workflow::{
    ReviewRequirementBinding, ReviewRequirementInput,
};

pub fn input(resource: &str, related: &str) -> ReviewRequirementInput {
    ReviewRequirementInput {
        resource: resource.to_owned(),
        related: related.to_owned(),
    }
}

/// Asserts that admission derived exactly the request's key and input
/// identities, streamed into their hashes with no basis sequence. Each hash
/// covers the encoded value plus its framing: the 30-byte domain and the scope,
/// each behind an 8-byte length. A hash of `n` input bytes takes
/// `ceil((n + 9) / 64)` SHA-256 compression blocks.
pub fn assert_request_identity_work(
    admission: WorthQueryCanonicalWorkEvidence,
    encoded: usize,
    hashed: usize,
    blocks: usize,
) {
    assert_eq!(admission.basis_preparations(), 0);
    assert_eq!(
        admission.digest_derivations(),
        2,
        "one key and one input derivation"
    );
    assert_eq!(admission.canonical_entries(), 2);
    assert_eq!(admission.canonical_encoded_bytes(), encoded);
    assert_eq!(admission.canonical_material_allocation_bytes(), 0);
    assert_eq!(admission.sha256_input_bytes(), hashed);
    assert_eq!(admission.sha256_compression_blocks(), blocks);
    assert_eq!(admission.digest_text_materializations(), 0);
}
