//! Post-performance snapshot closeout preserves both commit identity and refusal.
use super::RecoveredCheckpointTransitionError;
use crate::runtime::RelationalRuntime;
use crate::transactions::data::CommitResult;

pub(super) fn release(
    runtime: &RelationalRuntime,
    committed: &CommitResult,
) -> Result<(), RecoveredCheckpointTransitionError> {
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .map(|_| ())
        .map_err(
            |denial| RecoveredCheckpointTransitionError::SnapshotCloseoutFailed {
                commit_id: committed.commit.commit_id,
                denial,
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::support::*;
    #[test]
    fn an_already_closed_performed_snapshot_preserves_its_commit_and_typed_denial() {
        let host = crate::test_execution_policy::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let runtime = runtime_with_test_schema();
        let committed = create_entity_outcome(&execution, &runtime, "closeout-refusal");
        runtime
            .snapshots()
            .release_snapshot(&committed.snapshot)
            .unwrap();
        let Err(RecoveredCheckpointTransitionError::SnapshotCloseoutFailed { commit_id, denial }) =
            release(&runtime, &committed)
        else {
            panic!("snapshot closeout cannot be relabelled as a pre-effect refusal");
        };
        assert_eq!(commit_id, committed.commit.commit_id);
        assert_eq!(
            denial,
            crate::visibility::RelationalSnapshotReleaseDenial::UnknownSnapshot
        );
        assert!(runtime
            .history()
            .immutable_commit_receipt(commit_id)
            .is_some());
    }
}
