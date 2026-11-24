//! The removed binding keeps its real observation until owner retirement.
use super::selected_snapshot::VisibilitySnapshotBasis;
use crate::branch::root::RelationalBranchRoot;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct VisibilitySnapshotBasisRetirement {
    observation: Option<crate::mvcc::observation::RelationalObservationRetirement>,
    _root: Arc<RelationalBranchRoot>,
}

impl VisibilitySnapshotBasis {
    pub(crate) fn into_retirement(mut self) -> VisibilitySnapshotBasisRetirement {
        VisibilitySnapshotBasisRetirement {
            observation: self.take_observation_retirement(),
            _root: self.root,
        }
    }
}

impl VisibilitySnapshotBasisRetirement {
    pub(crate) fn advance(&mut self) -> Result<bool, ()> {
        match self.observation.as_mut() {
            Some(observation) => observation.advance(),
            None => Ok(true),
        }
    }
}
