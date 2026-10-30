pub(crate) use worth_query_testkit_certification_canonical::facade::certification::canonical_identity::*;

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
