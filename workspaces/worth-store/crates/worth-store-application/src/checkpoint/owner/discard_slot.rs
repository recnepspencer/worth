//! Exclusive ownership of the Home checkpoint slot during D14 derived discard.
use super::{CheckpointSlot, HomeCheckpointOwner};
use std::sync::{Arc, Mutex};
use worth_store::physical_runtime::AnchoredRecordPublicationFailureCustody;

/// Prevents any concurrent checkpoint installation, acknowledgement, or second
/// discard from interleaving with an in-flight derived-state discard.
pub(crate) struct DiscardSlotGuard<'a> {
    owner: &'a HomeCheckpointOwner,
    finished: bool,
}

impl<'a> DiscardSlotGuard<'a> {
    pub(super) fn new(owner: &'a HomeCheckpointOwner) -> Self {
        Self {
            owner,
            finished: false,
        }
    }

    pub(in crate::checkpoint) fn finish(mut self, published: bool) {
        let mut state = self
            .owner
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if published {
            state.acknowledged_native = [0; 2];
            state.derived_discarded = true;
        }
        if matches!(state.slot, CheckpointSlot::Discarding) {
            state.slot = CheckpointSlot::Idle;
        }
        self.finished = true;
    }

    pub(in crate::checkpoint) fn finish_unresolved(
        mut self,
        failure: AnchoredRecordPublicationFailureCustody,
    ) {
        let mut state = self
            .owner
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        state.slot = CheckpointSlot::DiscardUnresolved {
            _custody: Arc::new(Mutex::new(failure)),
        };
        self.finished = true;
    }
}

impl Drop for DiscardSlotGuard<'_> {
    fn drop(&mut self) {
        if !self.finished {
            let mut state = self
                .owner
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            if matches!(state.slot, CheckpointSlot::Discarding) {
                state.slot = CheckpointSlot::Idle;
            }
        }
    }
}
