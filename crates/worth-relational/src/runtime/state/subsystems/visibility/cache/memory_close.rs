//! Sealed owner traversal, retaining the original table and state allocations.
use super::*;

#[derive(Debug)]
pub(super) struct CachedSnapshotState {
    pub(super) state: Option<SnapshotState>,
    retirement: Option<crate::visibility::snapshot_states::SnapshotStateRetirement>,
}

impl CachedSnapshotState {
    pub(super) fn new(state: SnapshotState) -> Self {
        Self {
            state: Some(state),
            retirement: None,
        }
    }
}

impl VisibilityCache {
    /// Observe the actual original table under its nonwaiting owner gate.
    pub(crate) fn try_has_memory_states(&self) -> Result<bool, ()> {
        let states = match self.states.try_read() {
            Ok(states) => states,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(()),
        };
        Ok(!states.is_empty())
    }

    /// No allocation, rehash, shrink or ordinary payload Drop. Called only
    /// after authentic owner admission is stopped and all operations returned.
    pub(crate) fn try_retire_memory_state(&self, index: usize) -> Result<Option<bool>, ()> {
        let mut states = match self.states.try_write() {
            Ok(value) => value,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(()),
        };
        let Some((_, entry)) = states.get_index_mut(index) else {
            return Ok(None);
        };
        if let Some(state) = entry.state.take() {
            entry.retirement =
                Some(crate::visibility::snapshot_states::SnapshotStateRetirement::new(state));
            return Ok(Some(false));
        }
        entry.retirement.as_mut().expect(
            /* Proof: Only this sealed transition takes state; it installs retirement in the same exclusive table hold before returning. Ordinary lookup requires Some(state), and no stopped service can replace the entry. */
            "a consumed cache entry owns its actual retirement"
        ).advance().map(Some)
    }
}

#[cfg(test)]
impl VisibilityCache {
    pub(crate) fn memory_close_entry_count(&self) -> usize {
        self.states.read().unwrap().len()
    }

    pub(crate) fn while_live_entries_locked(&self, count: usize, action: impl FnOnce()) {
        let locked = self.states.write().unwrap();
        action();
        assert_eq!(locked.len(), count);
        assert!(locked
            .values()
            .all(|entry| entry.state.is_some() && entry.retirement.is_none()));
    }

    pub(crate) fn assert_entries_retired(&self, count: usize) -> VisibilitySnapshotStateKey {
        let states = self.states.read().unwrap();
        assert_eq!(states.len(), count, "no shrink/rehash/payload accumulation");
        assert!(states
            .values()
            .all(|entry| entry.state.is_none() && entry.retirement.is_some()));
        states.first().unwrap().0.clone()
    }
}
