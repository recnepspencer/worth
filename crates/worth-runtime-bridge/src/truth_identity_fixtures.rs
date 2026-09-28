use crate::identity::BridgeIdentityPayload;
use crate::input::envelope::{TruthBranchIdentity, TruthCommitIdentity, TruthPatchIdentity};
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
    let label = label.into();
    TruthCommitIdentity::with_payload(
        label.clone(),
        BridgeIdentityPayload::RelationalCommit {
            commit_id: fixture_position(label),
        },
    )
}

pub(crate) fn truth_patch_fixture(label: impl Into<String>) -> TruthPatchIdentity {
    let label = label.into();
    TruthPatchIdentity::with_payload(
        label.clone(),
        BridgeIdentityPayload::RelationalPatch {
            patch_position: fixture_position(label),
        },
    )
}

pub(crate) fn truth_snapshot_fixture(label: impl Into<String>) -> TruthSnapshotIdentity {
    let label = label.into();
    let position = fixture_position(&label);
    TruthSnapshotIdentity::with_payload(
        label,
        BridgeIdentityPayload::RelationalSnapshot {
            snapshot_id: position,
            version_id: 1,
        },
    )
}

pub(crate) fn truth_branch_fixture(label: impl Into<String>) -> TruthBranchIdentity {
    let label = label.into();
    TruthBranchIdentity::with_payload(
        label.clone(),
        BridgeIdentityPayload::RelationalBranch {
            branch_id: label.into(),
        },
    )
}

pub(crate) fn malformed_empty_truth_branch_for_validation() -> TruthBranchIdentity {
    TruthBranchIdentity::admit_bridge_owned("")
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

fn fixture_position(label: impl Into<String>) -> u64 {
    let label = label.into();
    if let Some(position) = fixture_suffix_position(&label) {
        return position;
    }
    label.bytes().fold(17_u64, |acc, byte| {
        acc.wrapping_mul(131).wrapping_add(u64::from(byte))
    })
}

fn fixture_suffix_position(label: &str) -> Option<u64> {
    if label.split(['-', ':']).count() != 2 {
        return None;
    }
    let suffix = label.rsplit(['-', ':']).next()?;
    match suffix {
        "a" => Some(1),
        "b" => Some(2),
        "c" => Some(3),
        "d" => Some(4),
        "e" => Some(5),
        "f" => Some(6),
        _ => suffix.parse::<u64>().ok(),
    }
}
