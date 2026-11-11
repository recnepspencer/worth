//! Real owner semantics are captured before sealing and recovered as data.
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact as Artifact;
use crate::tests::support::*;
use worth_execution::ExecutionRequest;

mod custom_versions;

#[test]
fn semantic_result_discovery_must_be_present_in_each_actual_index_entry() {
    use crate::history::RelationalCommitIndexEntry;
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let committed = create_entity_outcome(execution, &runtime, "mandatory-index-discovery");
    let artifact = runtime
        .history
        .commit_artifact(committed.commit.commit_id)
        .unwrap();
    let positioned = runtime
        .history
        .positioned_canonical_commits_snapshot()
        .pop()
        .unwrap();
    let original = runtime
        .durability()
        .canonical_commit_artifacts(execution)
        .next()
        .unwrap()
        .unwrap();
    let entry = crate::history::RetainedCanonicalIndex::from_publication(
        &positioned,
        artifact.roots(),
        *original.payload_digest().as_bytes(),
        execution,
    )
    .unwrap();
    assert_eq!(
        entry.association_discovery,
        crate::history::data::RelationalAssociationDiscovery::Unassociated
    );
    let mut wire = serde_json::to_value(entry.as_ref()).unwrap();
    assert_eq!(
        serde_json::from_value::<RelationalCommitIndexEntry>(wire.clone()).unwrap(),
        *entry
    );
    wire.as_object_mut()
        .unwrap()
        .remove("association_discovery")
        .unwrap();
    let failure = serde_json::from_value::<RelationalCommitIndexEntry>(wire).unwrap_err();
    assert!(
        failure.to_string().contains("association_discovery"),
        "{failure}"
    );
    release_test_commit_snapshot(&runtime, &committed);
}
