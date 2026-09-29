//! Canonical operation identity through real bindings and the ordinary
//! application request API.
//!
//! An author never writes an input identity: Query derives it from every field
//! of the serialized input. These proofs show what that buys a caller: text
//! that only moves a delimiter between fields is a different request, and any
//! changed field under a reused idempotency key is intent drift, never a replay.
//! A request encodes its input and its key once each, and its receipt reports
//! exactly those two derivations in admission, whether it commits or replays.

use worth_query_decl::facade::application_operation::ApplicationMutationIdentities;
use worth_query_host::facade::domain::WorthQueryCanonicalWorkEvidence;

use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
};

use crate::document_retention_model::host::{
    publish_on_first_program, publish_workflow_on_first_program, SEED_RETENTION,
};
use crate::document_retention_model::workflow::{publish_definition, reviewed_document_definition};

use crate::document_retention_model::presented_request::set_retention;
use crate::document_retention_model::readback::read_retention;
use crate::document_retention_model::schema::{DocumentRetentionSchema, SetRetentionInput};
use crate::document_retention_model::workflow::{ReviewRequirementBinding, ReviewRequirementInput};

fn input(resource: &str, related: &str) -> ReviewRequirementInput {
    ReviewRequirementInput {
        resource: resource.to_owned(),
        related: related.to_owned(),
    }
}

#[test]
fn a_delimiter_moved_between_string_fields_derives_a_different_input_identity() {
    let identity = |resource: &str, related: &str| {
        let key = 1_u64;
        *ApplicationMutationIdentities::<DocumentRetentionSchema, ReviewRequirementBinding>::encode(
            &key,
            &input(resource, related),
        )
        .expect("the request encodes")
        .input_identity()
    };
    assert_ne!(identity("a:b", "c"), identity("a", "b:c"));
    assert_eq!(identity("a:b", "c"), identity("a:b", "c"));
}

#[test]
fn a_changed_retention_under_a_reused_key_is_intent_drift_and_the_unchanged_retry_replays() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let key = 0x2701_0101;
    let first = SEED_RETENTION + 1;
    assert!(matches!(
        set_retention(&host, branch, first, key).expect("the first request settles"),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert!(
        matches!(
            set_retention(&host, branch, first, key).expect("the unchanged retry settles"),
            WorthQueryApplicationMutationOutcome::AlreadyCommitted(_)
        ),
        "the unchanged retry replays as already committed"
    );
    let drifted = set_retention(&host, branch, first + 1, key).expect("the drifted retry settles");
    assert!(matches!(
        drifted,
        WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
    ));
    assert_eq!(
        read_retention(host.runtime(), branch),
        first,
        "the drifted retry must not write"
    );
}

#[test]
fn a_request_encodes_its_input_once_whether_it_commits_or_replays() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let key = 0x2701_0102;
    let retention = SEED_RETENTION + 1;

    SetRetentionInput::reset_encoding_count();
    let first = set_retention(&host, branch, retention, key).expect("the first request settles");
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = first else {
        panic!("the first request commits: {first:?}");
    };
    assert_eq!(
        SetRetentionInput::encoding_count(),
        1,
        "a committing request encodes its input once for admission, execution and the commit"
    );
    // The key 0x2701_0102 (6 encoded bytes, 107 hashed with the 55-byte key
    // namespace, 2 blocks) and the input (60 encoded bytes, 163 hashed with the
    // 57-byte input type, 3 blocks).
    assert_request_identity_work(receipt.canonical_work().admission(), 66, 270, 5);

    SetRetentionInput::reset_encoding_count();
    let retry = set_retention(&host, branch, retention, key).expect("the retry settles");
    let WorthQueryApplicationMutationOutcome::AlreadyCommitted(recovered) = retry else {
        panic!("the unchanged retry replays: {retry:?}");
    };
    assert_eq!(
        SetRetentionInput::encoding_count(),
        1,
        "a replayed request encodes its input once too"
    );
    assert_request_identity_work(recovered.canonical_work().admission(), 66, 270, 5);
}

#[test]
fn a_capability_governed_request_encodes_its_input_once_whether_it_commits_or_replays() {
    let application = publish_workflow_on_first_program();
    let key = 0x2701_0103;
    let publish = || {
        publish_definition(
            &application,
            reviewed_document_definition("applied"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            key,
        )
        .expect("the capability-governed publication prepares")
    };

    SetRetentionInput::reset_encoding_count();
    let WorkflowDefinitionPublicationOutcome::Published(first) = publish() else {
        panic!("the first publication lands");
    };
    assert!(!first.replayed());
    assert_eq!(
        SetRetentionInput::encoding_count(),
        1,
        "the request's one input encoding is also the input its capability admission governs"
    );
    // The same input identity, and a key hashed under the 60-byte publication
    // key namespace: 112 hashed key bytes, still 2 blocks.
    assert_request_identity_work(first.receipt().canonical_work().admission(), 66, 275, 5);

    SetRetentionInput::reset_encoding_count();
    let WorkflowDefinitionPublicationOutcome::Published(replayed) = publish() else {
        panic!("the unchanged retry replays the publication");
    };
    assert!(replayed.replayed());
    assert_eq!(
        SetRetentionInput::encoding_count(),
        1,
        "a replay encodes its input once too"
    );
    assert_request_identity_work(replayed.receipt().canonical_work().admission(), 66, 275, 5);
}

/// Asserts that admission derived exactly the request's key and input
/// identities, streamed into their hashes with no basis sequence. Each hash
/// covers the encoded value plus its framing: the 30-byte domain and the scope,
/// each behind an 8-byte length. A hash of `n` input bytes takes
/// `ceil((n + 9) / 64)` SHA-256 compression blocks.
pub(crate) fn assert_request_identity_work(
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
