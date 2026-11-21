//! Projection transport preserves the distinction between committed index
//! generations and an actual applied, unpublished candidate.
use worth_relational::facade::{
    runtime::{
        RelationalAppliedCandidateReadIdentity, RelationalPinnedRead, VisibilityProjectionView,
    },
    snapshots::SnapshotHandle,
};

pub struct ResultTreeRead<'a> {
    pub projection: VisibilityProjectionView,
    pub committed_ordering: Option<&'a RelationalPinnedRead>,
}

impl<'a> ResultTreeRead<'a> {
    pub fn committed(read: &'a RelationalPinnedRead, snapshot: &SnapshotHandle) -> Option<Self> {
        Some(Self {
            projection: read.project_snapshot(snapshot)?,
            committed_ordering: Some(read),
        })
    }

    pub fn applied(read: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>) -> Self {
        Self {
            projection: read.project_candidate(),
            committed_ordering: None,
        }
    }
}
