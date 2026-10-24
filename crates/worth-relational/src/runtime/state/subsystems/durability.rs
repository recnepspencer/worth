use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::durability::data::{DurableCheckpoint, DurableStore};
use crate::durability::medium::{RelationalDurableMedium, RelationalDurableMediumBinding};
use crate::history::data::{CommitId, PositionedCanonicalCommit};
use crate::runtime::state::subsystems::{RuntimeOwnedState, RuntimeSubsystem};
use crate::runtime::RelationalRuntimeConfig;

mod state;

use state::DurabilityState;

/// The runtime's durable log and checkpoint authority, owned behind its own
/// lock so durability I/O never demands exclusive access to the runtime.
#[derive(Debug, Default)]
pub(crate) struct DurabilitySubsystem {
    state: RuntimeOwnedState<DurabilityState>,
    fail_next_append: AtomicBool,
    store_io: Arc<Mutex<()>>,
    /// The durable medium every commit is published to before it becomes
    /// live. A forked runtime is a speculative copy and never inherits it.
    medium: Option<Arc<RelationalDurableMediumBinding>>,
}

impl DurabilitySubsystem {
    pub(crate) fn attach_medium(&mut self, medium: Arc<dyn RelationalDurableMedium>) {
        self.medium = Some(Arc::new(RelationalDurableMediumBinding::new(medium)));
    }

    pub(crate) fn medium(&self) -> Option<Arc<RelationalDurableMediumBinding>> {
        self.medium.clone()
    }

    pub(crate) fn lock_store(&self) -> MutexGuard<'_, ()> {
        self.store_io
            .lock()
            .expect("native store I/O lock poisoned")
    }
    pub(crate) fn try_lock_store(&self) -> std::sync::TryLockResult<MutexGuard<'_, ()>> {
        self.store_io.try_lock()
    }
    pub(crate) fn push_log_envelope(&self, commit: PositionedCanonicalCommit) {
        self.state.write().push_log_envelope(commit);
    }

    pub(crate) fn push_checkpoint(&self, checkpoint: DurableCheckpoint) {
        self.state.write().push_checkpoint(checkpoint);
    }

    pub(crate) fn set_log(&self, log: Vec<PositionedCanonicalCommit>) {
        self.state.write().set_log(log);
    }

    #[cfg(test)]
    pub(crate) fn remove_log_commit(&self, commit_id: CommitId) -> bool {
        self.state.write().remove_log_commit(commit_id)
    }

    pub(crate) fn retain_log_after(&self, commit_id: CommitId) {
        self.state.write().retain_log_after(commit_id);
    }

    pub(crate) fn trim_log_front(&self, count: usize) {
        self.state.write().trim_log_front(count);
    }

    pub(crate) fn durable_log_envelope(
        &self,
        commit_id: CommitId,
    ) -> Option<Arc<PositionedCanonicalCommit>> {
        self.state.read().durable_log_envelope(commit_id)
    }

    pub(crate) fn checkpoint_envelope(
        &self,
        commit_id: CommitId,
    ) -> Option<PositionedCanonicalCommit> {
        self.state.read().checkpoint_envelope(commit_id)
    }

    /// Every durable envelope, shared rather than copied, so scans never hold
    /// the subsystem lock while they read.
    pub(crate) fn log(&self) -> Vec<Arc<PositionedCanonicalCommit>> {
        self.state.read().log.clone()
    }

    pub(crate) fn log_len(&self) -> usize {
        self.state.read().log.len()
    }

    pub(crate) fn checkpoints(&self) -> Vec<Arc<DurableCheckpoint>> {
        self.state.read().checkpoints.clone()
    }

    pub(crate) fn latest_checkpoint(&self) -> Option<Arc<DurableCheckpoint>> {
        self.state.read().checkpoints.last().map(Arc::clone)
    }

    pub(crate) fn store(&self) -> Option<Arc<DurableStore>> {
        self.state.read().store.clone()
    }

    pub(crate) fn set_store(&self, store: Option<DurableStore>) {
        self.state.write().store = store.map(Arc::new);
    }

    pub(crate) fn arm_append_failure_for_test(&self) {
        self.fail_next_append.store(true, Ordering::Release);
    }

    pub(crate) fn take_armed_append_failure_for_test(&self) -> bool {
        self.fail_next_append.load(Ordering::Acquire)
            && self.fail_next_append.swap(false, Ordering::AcqRel)
    }
}

impl RuntimeSubsystem for DurabilitySubsystem {
    type Config = RelationalRuntimeConfig;

    fn new(config: &Self::Config) -> Self {
        Self {
            state: RuntimeOwnedState::new(DurabilityState::build_from_config(config)),
            fail_next_append: AtomicBool::new(false),
            store_io: Arc::new(Mutex::new(())),
            medium: None,
        }
    }

    fn fork(&self) -> Self {
        Self {
            state: self.state.detached(),
            fail_next_append: AtomicBool::new(self.fail_next_append.load(Ordering::Acquire)),
            store_io: Arc::clone(&self.store_io),
            medium: None,
        }
    }
}
