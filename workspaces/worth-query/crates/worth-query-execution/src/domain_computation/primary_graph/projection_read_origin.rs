//! Retained data provenance distinguishes committed and unpublished roots.
use worth_relational::facade::{
    runtime::RelationalAppliedCandidateReadQualification, snapshots::SnapshotHandle,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectionReadOrigin {
    Committed(SnapshotHandle),
    Applied(RelationalAppliedCandidateReadQualification),
}
impl ProjectionReadOrigin {
    pub(super) fn runtime_instance_id(&self) -> u64 {
        match self {
            Self::Committed(s) => s.runtime_instance_id(),
            Self::Applied(q) => q.runtime_instance_id(),
        }
    }
    pub(super) fn committed_snapshot(&self) -> Option<&SnapshotHandle> {
        match self {
            Self::Committed(snapshot) => Some(snapshot),
            Self::Applied(_) => None,
        }
    }
}
impl PartialEq<SnapshotHandle> for ProjectionReadOrigin {
    fn eq(&self, other: &SnapshotHandle) -> bool {
        self.committed_snapshot() == Some(other)
    }
}

/// Borrowed provenance while an acquisition is in progress. Only a funded
/// retained result clones the committed snapshot's variable-width branch name.
#[derive(Clone, Copy)]
pub enum ProjectionReadOriginRef<'a> {
    Committed(&'a SnapshotHandle),
    Applied(RelationalAppliedCandidateReadQualification),
}
impl ProjectionReadOriginRef<'_> {
    pub(super) fn runtime_instance_id(&self) -> u64 {
        match self {
            Self::Committed(snapshot) => snapshot.runtime_instance_id(),
            Self::Applied(candidate) => candidate.runtime_instance_id(),
        }
    }

    pub(super) fn committed_snapshot(&self) -> Option<&SnapshotHandle> {
        match self {
            Self::Committed(s) => Some(s),
            Self::Applied(_) => None,
        }
    }
    pub(super) fn retained(self) -> ProjectionReadOrigin {
        match self {
            Self::Committed(s) => ProjectionReadOrigin::Committed(s.clone()),
            Self::Applied(q) => ProjectionReadOrigin::Applied(q),
        }
    }
}
impl PartialEq<ProjectionReadOriginRef<'_>> for ProjectionReadOrigin {
    fn eq(&self, other: &ProjectionReadOriginRef<'_>) -> bool {
        match (self, other) {
            (Self::Committed(a), ProjectionReadOriginRef::Committed(b)) => a == *b,
            (Self::Applied(a), ProjectionReadOriginRef::Applied(b)) => a == b,
            _ => false,
        }
    }
}

impl<'a> From<&'a SnapshotHandle> for ProjectionReadOriginRef<'a> {
    fn from(snapshot: &'a SnapshotHandle) -> Self {
        Self::Committed(snapshot)
    }
}

impl From<SnapshotHandle> for ProjectionReadOrigin {
    fn from(snapshot: SnapshotHandle) -> Self {
        Self::Committed(snapshot)
    }
}

mod court_doors_for_test;
