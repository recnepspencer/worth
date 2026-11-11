//! The removed binding keeps its real observation until owner retirement.
use super::state::VisibilitySnapshotBasis;

#[derive(Debug)]
pub(crate) struct VisibilitySnapshotBasisRetirement {
    observation: Option<crate::mvcc::RelationalObservationRetirement>,
    _root: std::sync::Arc<crate::branch::RelationalBranchRoot>,
}

impl VisibilitySnapshotBasis {
    pub(crate) fn into_retirement(self) -> VisibilitySnapshotBasisRetirement {
        VisibilitySnapshotBasisRetirement {
            observation: self
                ._retained_observation
                .map(|value| value.into_retirement()),
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
