//! A sealed cache entry keeps its allocation payload at its original owner.
use super::{SnapshotState, SnapshotStateBasis};

#[derive(Debug)]
pub(crate) struct SnapshotStateRetirement {
    _state: SnapshotState,
    exact: Option<crate::mvcc::RelationalObservationRetirement>,
    historical: Option<crate::history::retention::RelationalHistoricalObservationRetirement>,
}

impl SnapshotStateRetirement {
    pub(crate) fn new(mut state: SnapshotState) -> Self {
        let (exact, historical) = match &mut state.basis {
            SnapshotStateBasis::Exact(basis) => (
                basis
                    ._retained_observation
                    .take()
                    .map(|value| value.into_retirement()),
                None,
            ),
            SnapshotStateBasis::Historical(basis) => (None, basis.take_retirement()),
        };
        Self {
            _state: state,
            exact,
            historical,
        }
    }

    pub(crate) fn advance(&mut self) -> Result<bool, ()> {
        if let Some(value) = self.exact.as_mut() {
            return value.advance_retaining_allocation();
        }
        if let Some(value) = self.historical.as_mut() {
            return value.advance();
        }
        Ok(true)
    }
}
