use std::sync::Arc;

use crate::durability::data::DurableCheckpoint;
use crate::durability::medium::RelationalTransitionHistoryReader;
use crate::durability::medium::{RelationalDurableMedium, RelationalDurableMediumBinding};
use crate::history::data::{CommitId, PositionedCanonicalCommit};
use crate::runtime::state::subsystems::{RuntimeOwnedState, RuntimeSubsystem};
use crate::runtime::RelationalRuntimeConfig;

mod checkpoint_selection;
mod state;
pub(crate) use checkpoint_selection::{
    RelationalCheckpointManifestPublication, RelationalCheckpointManifestSelection,
};

use state::DurabilityState;

/// The runtime's canonical log and checkpoint authority, owned behind its own
/// lock, plus its attachment to a durable medium when it has one.
#[derive(Debug, Default)]
pub(crate) struct DurabilitySubsystem {
    state: RuntimeOwnedState<DurabilityState>,
    /// The durable medium every commit is published to before it becomes
    /// live. A forked runtime is a speculative copy and never inherits it.
    medium: Option<Arc<RelationalDurableMediumBinding>>,
    /// Speculative copies retain historical reads, never publication access.
    transition_history: Option<RelationalTransitionHistoryReader>,
    cold_source: Option<Arc<dyn crate::durability::medium::RelationalDurableColdSource>>,
    checkpoint_selection: RelationalCheckpointManifestSelection,
}

impl DurabilitySubsystem {
    pub(crate) fn checkpoint_selection(&self) -> &RelationalCheckpointManifestSelection {
        &self.checkpoint_selection
    }

    pub(crate) fn cold_source(
        &self,
    ) -> Option<Arc<dyn crate::durability::medium::RelationalDurableColdSource>> {
        self.cold_source.clone()
    }

    pub(crate) fn attach_cold_source(
        &mut self,
        source: Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    ) {
        self.cold_source = Some(source);
    }

    pub(crate) fn attach_medium(&mut self, medium: Box<dyn RelationalDurableMedium>) {
        let binding = Arc::new(RelationalDurableMediumBinding::new(medium));
        self.cold_source = Some(binding.cold_source());
        self.medium = Some(binding);
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

    pub(crate) fn transition_history(&self) -> Option<RelationalTransitionHistoryReader> {
        self.medium
            .as_ref()
            .map(|medium| medium.transition_history())
            .or_else(|| self.transition_history.clone())
    }

    pub(crate) fn push_log_envelope(
        &self,
        commit: (
            CommitId,
            crate::publication::patch::data::PatchStreamPosition,
            Arc<crate::history::RelationalCommitEnvelopeResidency>,
        ),
    ) {
        self.state.write().push_log_envelope(commit);
    }

    pub(crate) fn push_checkpoint(&self, checkpoint: DurableCheckpoint) {
        self.state.write().push_checkpoint(checkpoint);
    }

    pub(crate) fn set_log(
        &self,
        log: Vec<(
            CommitId,
            crate::publication::patch::data::PatchStreamPosition,
            Arc<crate::history::RelationalCommitEnvelopeResidency>,
        )>,
    ) {
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
        self.state
            .read()
            .log
            .iter()
            .filter_map(|entry| entry.2.positioned_if_resident())
            .map(Arc::new)
            .collect()
    }

    pub(crate) fn log_residencies_after(
        &self,
        after: Option<crate::publication::patch::data::PatchStreamPosition>,
    ) -> Vec<Arc<crate::history::RelationalCommitEnvelopeResidency>> {
        self.state
            .read()
            .log
            .iter()
            .filter(|entry| after.is_none_or(|position| entry.1 > position))
            .map(|entry| entry.2.clone())
            .collect()
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
            transition_history: None,
            cold_source: None,
            checkpoint_selection: Default::default(),
        }
    }

    fn fork(&self) -> Self {
        Self {
            state: self.state.detached(),
            medium: None,
            transition_history: self.transition_history(),
            cold_source: self.cold_source.clone(),
            checkpoint_selection: self.checkpoint_selection.detached(),
        }
    }
}
