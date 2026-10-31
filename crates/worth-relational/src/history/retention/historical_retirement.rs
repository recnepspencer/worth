//! Owned retirement of one historical observation share, with no basis lease.
use super::RelationalRetainedHistoricalRoot;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct RelationalHistoricalObservationRetirement {
    shared: Option<Arc<RelationalRetainedHistoricalRoot>>,
    exclusive: Option<RelationalRetainedHistoricalRoot>,
}

impl RelationalHistoricalObservationRetirement {
    pub(crate) fn new(shared: Arc<RelationalRetainedHistoricalRoot>) -> Self {
        Self {
            shared: Some(shared),
            exclusive: None,
        }
    }

    /// The authentic owner retains its live/retired root inventory throughout.
    /// Keep exclusive root/allocation payload here even after disarming custody.
    pub(crate) fn advance(&mut self) -> Result<bool, ()> {
        if let Some(shared) = self.shared.take() {
            self.exclusive = Arc::into_inner(shared);
            return Ok(self.exclusive.is_none());
        }
        match self.exclusive.as_mut() {
            Some(value) => value.try_retire_observation().then_some(true).ok_or(()),
            None => Ok(true),
        }
    }
}
