//! Sealed owner traversal, retaining the original table and state allocations.
use super::*;
#[cfg(test)]
mod tests;

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
