//! One real data image for observing a retained or freshly sealed fact.
use worth_relational::facade::{
    runtime::{
        RelationalAppliedCandidateReadIdentity, RelationalPinnedRead, RelationalRuntime,
        VisibilityProjectionView,
    },
    snapshots::SnapshotHandle,
};

#[derive(Clone, Copy)]
pub enum FactObservationBasis<'a> {
    Committed {
        runtime: &'a RelationalRuntime,
        snapshot: &'a SnapshotHandle,
    },
    Applied(&'a RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>),
}
impl FactObservationBasis<'_> {
    pub fn projection(self) -> Option<VisibilityProjectionView> {
        match self {
            Self::Committed { runtime, snapshot } => {
                runtime.read_truth().project_snapshot(snapshot)
            }
            Self::Applied(pin) => Some(pin.project_candidate()),
        }
    }
}
