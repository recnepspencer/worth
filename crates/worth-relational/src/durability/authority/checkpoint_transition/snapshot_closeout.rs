//! Post-performance snapshot closeout preserves both commit identity and refusal.
use crate::history::data::CommitId;
use crate::visibility::RelationalSnapshotReleaseDenial;

pub(super) struct SnapshotCloseoutFailure {
    pub(super) commit_id: CommitId,
    pub(super) denial: RelationalSnapshotReleaseDenial,
}

pub(super) fn closeout(
    commit_id: CommitId,
    release: Result<(), RelationalSnapshotReleaseDenial>,
) -> Result<(), SnapshotCloseoutFailure> {
    release.map_err(|denial| SnapshotCloseoutFailure { commit_id, denial })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::support::*;
    #[test]
    fn an_already_closed_performed_snapshot_preserves_its_commit_and_typed_denial() {
        let host = crate::execution::test_policy::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let runtime = runtime_with_test_schema();
        let committed = create_entity_outcome(&execution, &runtime, "closeout-refusal");
        runtime
            .snapshots()
            .release_snapshot(&committed.snapshot)
            .unwrap();
        let release = runtime.snapshots().release_snapshot(&committed.snapshot);
        let Err(SnapshotCloseoutFailure { commit_id, denial }) =
            closeout(committed.commit.commit_id, release.map(|_| ()))
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
