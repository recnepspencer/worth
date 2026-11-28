use crate::input::envelope::{TruthBranchIdentity, TruthCommitIdentity, TruthPatchIdentity};
use crate::relational_source::identity_parts::fixture_position;
use crate::relational_source::identity_parts::RelationalBridgeSnapshotIdentityParts;
use crate::snapshot::TruthSnapshotIdentity;

pub(crate) fn truth_branch(label: &'static str) -> TruthBranchIdentity {
    TruthBranchIdentity::from_relational_branch_id(label)
}

pub(crate) fn truth_commit(position: u64) -> TruthCommitIdentity {
    TruthCommitIdentity::from_relational_commit_id(position)
}

pub(crate) fn truth_patch(position: u64) -> TruthPatchIdentity {
    TruthPatchIdentity::from_relational_patch_position(position)
}

pub(crate) fn truth_snapshot(snapshot_id: u64, version_id: u64) -> TruthSnapshotIdentity {
    TruthSnapshotIdentity::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts::new(
        snapshot_id,
        version_id,
    ))
}

pub(crate) fn truth_commit_fixture(label: impl Into<String>) -> TruthCommitIdentity {
    TruthCommitIdentity::from_bridge_harness_label(label)
}

pub(crate) fn truth_patch_fixture(label: impl Into<String>) -> TruthPatchIdentity {
    TruthPatchIdentity::from_bridge_harness_label(label)
}

pub(crate) fn truth_snapshot_fixture(label: impl Into<String>) -> TruthSnapshotIdentity {
    TruthSnapshotIdentity::from_bridge_harness_label(label)
}

pub(crate) fn truth_branch_fixture(label: impl Into<String>) -> TruthBranchIdentity {
    TruthBranchIdentity::from_relational_branch_id(label)
}

pub(crate) fn malformed_empty_truth_branch_for_validation() -> TruthBranchIdentity {
    TruthBranchIdentity::from_relational_branch_id("")
}

pub(crate) fn truth_commit_fixture_matches(identity: &TruthCommitIdentity, label: &str) -> bool {
    identity
        .relational_commit_id()
        .is_some_and(|commit_id| commit_id == fixture_position(label))
}

pub(crate) fn truth_snapshot_fixture_matches(
    identity: &TruthSnapshotIdentity,
    label: &str,
) -> bool {
    identity.relational_snapshot_parts().is_some_and(|parts| {
        parts.snapshot_id() == fixture_position(label) && parts.version_id() == 1
    })
}
