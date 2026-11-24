//! Sealed owner traversal, retaining the original table and state allocations.
use super::*;
use crate::history::retention::RetentionBusy;

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
    pub fn try_has_memory_states(&self) -> Result<bool, RetentionBusy> {
        let states = match self.states.try_read() {
            Ok(states) => states,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(RetentionBusy),
        };
        Ok(!states.is_empty())
    }

    /// No allocation, rehash, shrink or ordinary payload Drop. Called only
    /// after authentic owner admission is stopped and all operations returned.
    pub fn try_retire_memory_state(&self, index: usize) -> Result<Option<bool>, RetentionBusy> {
        let mut states = match self.states.try_write() {
            Ok(value) => value,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(RetentionBusy),
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

impl VisibilityCache {
    pub(crate) fn memory_close_entry_count(&self) -> usize {
        self.states.read().unwrap().len()
    }

    #[expect(
        clippy::readonly_write_lock,
        reason = "Exclusive custody lets the fixture hold both read and write contenders busy"
    )]
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

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait VisibilityCacheFixturePort: Sized {
    fn assert_entries_retired_for_test(&self, count: usize) -> VisibilitySnapshotStateKey;
    fn memory_close_entry_count_for_test(&self) -> usize;
    fn while_live_entries_locked_for_test(&self, count: usize, action: impl FnOnce());
}

impl VisibilityCacheFixturePort for VisibilityCache {
    fn assert_entries_retired_for_test(&self, count: usize) -> VisibilitySnapshotStateKey {
        self.assert_entries_retired(count)
    }

    fn memory_close_entry_count_for_test(&self) -> usize {
        self.memory_close_entry_count()
    }

    fn while_live_entries_locked_for_test(&self, count: usize, action: impl FnOnce()) {
        self.while_live_entries_locked(count, action)
    }
}
