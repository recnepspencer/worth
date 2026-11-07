//! Real owner semantics are captured before sealing and recovered as data.
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact as Artifact;
use crate::tests::support::*;
use crate::transactions::data::{
    CreatedEntityRef, EntityReference, RelationalCreatedEndpoint as Endpoint,
};
use worth_execution::ExecutionRequest;

mod bulk;
mod custom_versions;

#[test]
fn semantic_result_discovery_must_be_present_in_each_actual_index_entry() {
    use crate::history::RelationalCommitIndexEntry;
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let committed = create_entity_outcome(execution, &runtime, "mandatory-index-discovery");
    let entry = runtime
        .history
        .commit_artifact(committed.commit.commit_id)
        .unwrap()
        .envelope_residency()
        .journal_index()
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
