use std::sync::Arc;

use crate::durability::data::DurableCheckpoint;
use crate::durability::medium::{RelationalDurableMedium, RelationalDurableMediumBinding};
use crate::history::data::{CommitId, PositionedCanonicalCommit};
use crate::runtime::state::subsystems::{RuntimeOwnedState, RuntimeSubsystem};
use crate::runtime::RelationalRuntimeConfig;

mod state;

use state::DurabilityState;

/// The runtime's canonical log and checkpoint authority, owned behind its own
/// lock, plus its attachment to a durable medium when it has one.
#[derive(Debug, Default)]
pub(crate) struct DurabilitySubsystem {
    state: RuntimeOwnedState<DurabilityState>,
    /// The durable medium every commit is published to before it becomes
    /// live. A forked runtime is a speculative copy and never inherits it.
    medium: Option<Arc<RelationalDurableMediumBinding>>,
}

impl DurabilitySubsystem {
    pub(crate) fn attach_medium(&mut self, medium: Box<dyn RelationalDurableMedium>) {
        self.medium = Some(Arc::new(RelationalDurableMediumBinding::new(medium)));
    }

    /// Move the medium out so it can move into the runtime recovery rebuilt.
    /// This runtime, and every port that holds its attachment, then refuses
    /// to publish. `None` when it has no medium or the medium already moved.
    pub(crate) fn detach_medium(&self) -> Option<Box<dyn RelationalDurableMedium>> {
        self.medium.as_ref().and_then(|binding| binding.detach())
    }

    pub(crate) fn medium(&self) -> Option<Arc<RelationalDurableMediumBinding>> {
        self.medium.clone()
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
}

impl RuntimeSubsystem for DurabilitySubsystem {
    type Config = RelationalRuntimeConfig;

    fn new(_config: &Self::Config) -> Self {
        Self {
            state: RuntimeOwnedState::new(DurabilityState::default()),
            medium: None,
        }
    }

    fn fork(&self) -> Self {
        Self {
            state: self.state.detached(),
            medium: None,
        }
    }
}
