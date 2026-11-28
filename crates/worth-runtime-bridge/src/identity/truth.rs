use std::fmt;
use std::sync::Arc;

use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::identity::VersionId;
use worth_relational::facade::publication::PatchStreamPosition;
use worth_relational::facade::snapshots::SnapshotId;

use crate::clone_budget::CheapClone;
use crate::relational_source::identity_parts::fixture_position;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct OwnedSignalPublication {
    graph: u64,
    publication: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum PublicationCoordinate<T> {
    Relational(T),
    OwnedSignal(OwnedSignalPublication),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TruthCommitIdentity(PublicationCoordinate<CommitId>);

impl TruthCommitIdentity {
    pub const fn relational(commit: CommitId) -> Self {
        Self(PublicationCoordinate::Relational(commit))
    }

    pub const fn commit_id(self) -> Option<CommitId> {
        match self.0 {
            PublicationCoordinate::Relational(commit) => Some(commit),
            PublicationCoordinate::OwnedSignal(_) => None,
        }
    }

    pub const fn from_relational_commit_id(commit: u64) -> Self {
        Self::relational(CommitId(commit))
    }

    pub fn relational_commit_id(&self) -> Option<u64> {
        self.commit_id().map(|commit| commit.0)
    }

    pub fn from_bridge_harness_label(label: impl Into<String>) -> Self {
        Self::from_relational_commit_id(fixture_position(label))
    }

    pub(crate) const fn owned_signal(graph: u64, publication: u64) -> Self {
        Self(PublicationCoordinate::OwnedSignal(OwnedSignalPublication {
            graph,
            publication,
        }))
    }

    pub(crate) fn canonical_label(&self) -> String {
        match self.0 {
            PublicationCoordinate::Relational(CommitId(commit)) => {
                format!("relational-commit:{commit}")
            }
            PublicationCoordinate::OwnedSignal(OwnedSignalPublication { graph, publication }) => {
                format!("commit:owned-semantic:{graph}:{publication}")
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TruthPatchIdentity(PublicationCoordinate<PatchStreamPosition>);

impl TruthPatchIdentity {
    pub const fn relational(position: PatchStreamPosition) -> Self {
        Self(PublicationCoordinate::Relational(position))
    }

    pub const fn from_relational_patch_position(position: u64) -> Self {
        Self::relational(PatchStreamPosition(position))
    }

    pub const fn relational_patch_position(self) -> Option<u64> {
        match self.0 {
            PublicationCoordinate::Relational(PatchStreamPosition(position)) => Some(position),
            PublicationCoordinate::OwnedSignal(_) => None,
        }
    }

    pub fn from_bridge_harness_label(label: impl Into<String>) -> Self {
        Self::from_relational_patch_position(fixture_position(label))
    }

    pub(crate) const fn owned_signal(graph: u64, publication: u64) -> Self {
        Self(PublicationCoordinate::OwnedSignal(OwnedSignalPublication {
            graph,
            publication,
        }))
    }

    pub(crate) fn canonical_label(&self) -> String {
        match self.0 {
            PublicationCoordinate::Relational(PatchStreamPosition(position)) => {
                format!("relational-patch:{position}")
            }
            PublicationCoordinate::OwnedSignal(OwnedSignalPublication { graph, publication }) => {
                format!("patch:owned-semantic:{graph}:{publication}")
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TruthBranchIdentity(BranchCoordinate);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum BranchCoordinate {
    Relational(Arc<BranchId>),
    OwnedSignal,
}

impl TruthBranchIdentity {
    pub fn relational(branch: BranchId) -> Self {
        Self(BranchCoordinate::Relational(Arc::new(branch)))
    }

    pub fn branch_id(&self) -> Option<&BranchId> {
        match &self.0 {
            BranchCoordinate::Relational(branch) => Some(branch),
            BranchCoordinate::OwnedSignal => None,
        }
    }

    pub fn from_relational_branch_id(branch: impl Into<String>) -> Self {
        Self::relational(BranchId(branch.into()))
    }

    pub fn relational_branch_id(&self) -> Option<&str> {
        self.branch_id().map(|branch| branch.0.as_str())
    }

    pub fn from_bridge_harness_label(label: impl Into<Arc<str>>) -> Self {
        Self::from_relational_branch_id(label.into().as_ref())
    }

    pub(crate) const fn owned_signal() -> Self {
        Self(BranchCoordinate::OwnedSignal)
    }

    /// Formats this coordinate for reporting and canonical trace encoding.
    pub fn terminal_projection_for_reporting(&self) -> String {
        self.canonical_label()
    }

    pub(crate) fn canonical_label(&self) -> String {
        match &self.0 {
            BranchCoordinate::Relational(branch) => format!("relational-branch:{}", branch.0),
            BranchCoordinate::OwnedSignal => "branch:worth-ui-presentation".to_owned(),
        }
    }
}

/// Coordinates of a retained native snapshot. Neither coordinate grants read custody.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationalBridgeSnapshotIdentityParts {
    pub(crate) snapshot: SnapshotId,
    pub(crate) version: VersionId,
    pub(crate) runtime: Option<u64>,
}

impl RelationalBridgeSnapshotIdentityParts {
    pub const fn new(snapshot_id: u64, version_id: u64) -> Self {
        Self {
            snapshot: SnapshotId(snapshot_id),
            version: VersionId(version_id),
            runtime: None,
        }
    }

    pub const fn snapshot_id(self) -> u64 {
        self.snapshot.0
    }

    pub const fn version_id(self) -> u64 {
        self.version.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TruthSnapshotIdentity(SnapshotCoordinate);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SnapshotCoordinate {
    Retained(RelationalBridgeSnapshotIdentityParts),
    PinnedCommit { commit: CommitId, root: u64 },
    OwnedSignal(OwnedSignalPublication),
}

impl TruthSnapshotIdentity {
    pub const fn retained(snapshot: SnapshotId, version: VersionId) -> Self {
        Self::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts {
            snapshot,
            version,
            runtime: None,
        })
    }

    pub(crate) const fn retained_by(
        runtime: u64,
        snapshot: SnapshotId,
        version: VersionId,
    ) -> Self {
        Self::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts {
            snapshot,
            version,
            runtime: Some(runtime),
        })
    }

    pub const fn from_relational_snapshot(parts: RelationalBridgeSnapshotIdentityParts) -> Self {
        Self(SnapshotCoordinate::Retained(parts))
    }

    pub const fn relational_snapshot_parts(&self) -> Option<RelationalBridgeSnapshotIdentityParts> {
        match self.0 {
            SnapshotCoordinate::Retained(parts) => Some(parts),
            _ => None,
        }
    }

    pub fn from_bridge_harness_label(label: impl Into<String>) -> Self {
        Self::retained(SnapshotId(fixture_position(label)), VersionId(1))
    }

    pub(crate) const fn pinned_commit(commit: CommitId, root: u64) -> Self {
        Self(SnapshotCoordinate::PinnedCommit { commit, root })
    }

    pub(crate) const fn owned_signal(graph: u64, publication: u64) -> Self {
        Self(SnapshotCoordinate::OwnedSignal(OwnedSignalPublication {
            graph,
            publication,
        }))
    }

    /// Formats this coordinate for reporting and canonical trace encoding.
    pub fn terminal_projection_for_reporting(&self) -> String {
        self.canonical_label()
    }

    pub(crate) fn canonical_label(&self) -> String {
        match self.0 {
            SnapshotCoordinate::Retained(parts) => {
                format!(
                    "relational-snapshot:{}:version:{}",
                    parts.snapshot.0, parts.version.0
                )
            }
            SnapshotCoordinate::PinnedCommit { commit, root } => {
                format!("relational-pinned-commit:{}:{root}", commit.0)
            }
            SnapshotCoordinate::OwnedSignal(OwnedSignalPublication { graph, publication }) => {
                format!("snapshot:owned-semantic:{graph}:{publication}")
            }
        }
    }
}

macro_rules! opaque_truth_identity {
    ($($identity:ident),+ $(,)?) => {$(
        impl CheapClone for $identity {}

        impl fmt::Debug for $identity {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple(stringify!($identity)).field(&"<opaque>").finish()
            }
        }
    )+};
}

opaque_truth_identity!(
    TruthCommitIdentity,
    TruthPatchIdentity,
    TruthBranchIdentity,
    TruthSnapshotIdentity
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truth_identity_debug_redacts_native_coordinates() {
        assert_eq!(
            format!("{:?}", TruthCommitIdentity::relational(CommitId(7))),
            "TruthCommitIdentity(\"<opaque>\")",
        );
        assert_eq!(
            format!(
                "{:?}",
                TruthBranchIdentity::relational(BranchId("sealed".into()))
            ),
            "TruthBranchIdentity(\"<opaque>\")",
        );
        assert_eq!(
            format!(
                "{:?}",
                TruthSnapshotIdentity::retained(SnapshotId(3), VersionId(8))
            ),
            "TruthSnapshotIdentity(\"<opaque>\")",
        );
    }
}
