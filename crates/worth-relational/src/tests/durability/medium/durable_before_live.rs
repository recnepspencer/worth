//! A commit reaches the medium before it reaches live state, and a refusal
//! from the medium leaves live state exactly as it was.

use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::publication::patch::data::PatchStreamPosition;
use crate::tests::support::*;

#[test]
fn every_acknowledged_commit_is_the_artifact_the_medium_holds_at_its_position() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = durable_runtime_with_test_schema();
    let medium = attached_test_medium(&runtime);

    for name in ["first", "second", "third"] {
        let committed = create_entity_outcome(execution, &runtime, name);
        let held = medium
            .artifact_bytes(committed.patch_position())
            .expect("an acknowledged commit is already durable");
        let readmitted = RelationalCanonicalCommitArtifact::readmit(&held, execution)
            .expect("the medium holds Relational's own artifact");
        assert_eq!(readmitted.commit_id(), committed.commit.commit_id);
        assert_eq!(readmitted.position(), committed.patch_position());
        release_test_commit_snapshot(&runtime, &committed);
    }
    assert_eq!(
        medium.positions(),
        vec![
            PatchStreamPosition(1),
            PatchStreamPosition(2),
            PatchStreamPosition(3)
        ]
    );
}

#[test]
fn a_memory_only_runtime_publishes_without_a_medium() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_test_schema();

    let committed = create_entity_outcome(execution, &runtime, "memory-only");

    assert!(attached_durable_medium(&runtime).is_none());
    assert_eq!(runtime.durability().durable_commit_count(), 1);
    release_test_commit_snapshot(&runtime, &committed);
}
