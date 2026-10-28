//! A runtime recovered from a medium continues publishing to that medium at
//! the next position, and recovery itself never publishes.

use crate::facade::history::BranchId;
use crate::publication::patch::data::PatchStreamPosition;
use crate::tests::support::*;

#[test]
fn a_recovered_runtime_publishes_its_next_commit_at_the_next_position() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = durable_runtime_with_test_schema();
    let mut commit_ids = Vec::new();
    for name in ["a", "b"] {
        let committed = create_entity_outcome(execution, &runtime, name);
        commit_ids.push(committed.commit.commit_id);
        release_test_commit_snapshot(&runtime, &committed);
    }
    checkpoint_durably(execution, &runtime).unwrap();
    let tail = create_entity_outcome(execution, &runtime, "c");
    commit_ids.push(tail.commit.commit_id);
    release_test_commit_snapshot(&runtime, &tail);
    let medium = attached_test_medium(&runtime);
    let offered_before_recovery = medium.offered().len();

    let (recovery, recovered) = recover_with(execution, &runtime, durable_runtime_with_test_schema);
    let cold = recovered
        .durability
        .cold_source()
        .expect("recovery carries its read-only source");
    assert_eq!(
        cold.read_commit(PatchStreamPosition(1), 1 << 24).unwrap(),
        medium
            .artifact_bytes(PatchStreamPosition(1))
            .unwrap()
            .into_vec()
    );
    assert!(matches!(
        cold.read_commit(PatchStreamPosition(1), 1),
        Err(
            crate::storage::data::RelationalStorageUnavailableCause::LimitExceeded { limit: 1, .. }
        )
    ));

    assert_eq!(medium.offered().len(), offered_before_recovery);
    assert_eq!(
        recovery.coverage.checkpoint_covered_position,
        Some(PatchStreamPosition(2))
    );
    assert_eq!(recovery.coverage.replayed_tail_commits, 1);
    let next = create_entity_outcome(execution, &recovered, "d");
    assert_eq!(next.patch_position(), PatchStreamPosition(4));
    assert_eq!(next.commit.parents, vec![tail.commit.commit_id]);
    assert!(!commit_ids.contains(&next.commit.commit_id));
    assert_eq!(medium.head(), PatchStreamPosition(4));
    release_test_commit_snapshot(&recovered, &next);

    let (second, reopened_again) =
        recover_with(execution, &recovered, durable_runtime_with_test_schema);
    assert_eq!(second.recovered_commits, 4);
    let reopened_cold = reopened_again.durability.cold_source().unwrap();
    assert!(std::sync::Arc::ptr_eq(&cold, &reopened_cold));
    assert_eq!(
        reopened_again
            .history()
            .branch_head(&BranchId("main".to_owned())),
        Some(next.commit.clone())
    );
}
