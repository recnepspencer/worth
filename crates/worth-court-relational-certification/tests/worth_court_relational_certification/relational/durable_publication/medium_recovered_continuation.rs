//! Recovered owners continue the actual original medium without republishing history.
use super::branch_history_recovery::current_head;
use super::*;
use worth_relational::facade::{history::BranchId, publication::PatchStreamPosition};
#[test]
fn a_recovered_runtime_publishes_its_next_commit_at_the_next_position() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let mut commit_ids = Vec::new();
    for name in ["a", "b"] {
        let committed = create_entity_outcome(execution, &runtime, name);
        commit_ids.push(committed.commit.commit_id);
        release_test_commit_snapshot(&runtime, &committed);
    }
    super::checkpoint_source_capture::capture_durably(execution, &runtime, &medium).unwrap();
    let tail = create_entity_outcome(execution, &runtime, "c");
    commit_ids.push(tail.commit.commit_id);
    release_test_commit_snapshot(&runtime, &tail);
    let offered_before_recovery = medium.offered().len();

    let (recovery, recovered) =
        super::branch_history_recovery::recover(execution, &medium, durable_runtime(&medium));
    let cold: std::sync::Arc<
        dyn worth_relational::facade::durability::RelationalDurableColdSource,
    > = medium.clone();
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
            worth_relational::facade::storage::RelationalStorageUnavailableCause::LimitExceeded {
                limit: 1,
                ..
            }
        )
    ));

    let restored_frame = recovered
        .durability()
        .canonical_commit_artifacts(execution)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .into_iter()
        .find(|artifact| artifact.position() == PatchStreamPosition(1))
        .unwrap();
    assert_eq!(
        restored_frame.as_bytes(),
        medium
            .artifact_bytes(PatchStreamPosition(1))
            .unwrap()
            .as_ref()
    );
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
    assert_eq!(
        medium
            .committed_artifacts()
            .unwrap()
            .last()
            .unwrap()
            .position(),
        PatchStreamPosition(4)
    );
    release_test_commit_snapshot(&recovered, &next);

    let (second, reopened_again) =
        super::branch_history_recovery::recover(execution, &medium, durable_runtime(&medium));
    assert_eq!(second.recovered_commits, 4);
    assert_eq!(
        current_head(&reopened_again, &BranchId("main".to_owned())),
        Some(next.commit.clone())
    );
}
