//! A sealed cache entry keeps its allocation payload at its original owner.
use super::{SnapshotState, SnapshotStateBasis};
use crate::history::retention::RetentionBusy;

#[derive(Debug)]
pub struct SnapshotStateRetirement {
    _state: SnapshotState,
    exact: Option<crate::mvcc::observation::RelationalObservationRetirement>,
    historical: Option<crate::history::retention::RelationalHistoricalObservationRetirement>,
}

impl SnapshotStateRetirement {
    pub fn new(mut state: SnapshotState) -> Self {
        let (exact, historical) = match &mut state.basis {
            SnapshotStateBasis::Exact(basis) => (basis.take_observation_retirement(), None),
            SnapshotStateBasis::Historical(basis) => (None, basis.take_retirement()),
        };
        Self {
            _state: state,
            exact,
            historical,
        }
    }

    pub fn advance(&mut self) -> Result<bool, RetentionBusy> {
        if let Some(value) = self.exact.as_mut() {
            return value.advance_retaining_allocation();
        }
        if let Some(value) = self.historical.as_mut() {
            return value.advance();
        }
        Ok(true)
    }
}
