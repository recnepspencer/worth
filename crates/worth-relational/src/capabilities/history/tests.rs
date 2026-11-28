use crate::publication::patch::data::stream_commit_ref::PatchStreamCommitRef;
use crate::tests::support::{create_entity_outcome, runtime_with_test_schema};
use std::collections::BTreeSet;
use worth_execution::ExecutionRequest;

#[test]
fn history_routes_committed_envelopes_and_excludes_closure_members() {
    let runtime = runtime_with_test_schema();
    let serial = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&serial);
    let first = create_entity_outcome(&execution, &runtime, "first-history-member");
    let second = create_entity_outcome(&execution, &runtime, "second-history-member");
    let first = first.publication().envelope.commit.clone();
    let second = second.publication().envelope.commit.clone();
    let last_position = runtime
        .latest_patch_stream_position()
        .expect("second route");
    let envelope = runtime
        .commit_envelope_at_patch_stream_position(last_position)
        .expect("resident second envelope");
    assert_eq!(envelope.commit.commit_id, second.commit_id);

    assert!(runtime.has_committed_version_at_or_before_outside_closure(
        second.version_id,
        &BTreeSet::from([second.commit_id]),
    ));
    assert!(!runtime.has_committed_version_at_or_before_outside_closure(
        second.version_id,
        &BTreeSet::from([first.commit_id, second.commit_id]),
    ));

    let first_position = runtime
        .patch_stream_commits_after(None, 1)
        .first()
        .expect("first route")
        .position;
    assert!(runtime.contains_patch_stream_position(first_position));
    assert_eq!(
        runtime.patch_stream_commits_after(Some(first_position), 8),
        vec![PatchStreamCommitRef {
            position: last_position,
            commit_id: second.commit_id,
        }],
    );
}
