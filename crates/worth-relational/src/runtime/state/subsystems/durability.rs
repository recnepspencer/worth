use std::sync::Arc;

use crate::durability::data::DurableCheckpoint;
use crate::durability::published_medium::RelationalTransitionHistoryReader;
use crate::durability::published_medium::{
    RelationalDurableMedium, RelationalDurableMediumBinding,
};

use crate::history::data::{CommitId, PositionedCanonicalCommit};
use crate::runtime::state::subsystems::{RuntimeOwnedState, RuntimeSubsystem};
use crate::runtime::RelationalRuntimeConfig;

pub(crate) mod checkpoint_selection;
pub(crate) mod commit_log;
mod source_retirement;
mod state;
pub(crate) use checkpoint_selection::RelationalCheckpointManifestSelection;

use state::DurabilityState;

/// The runtime's canonical log and checkpoint authority, owned behind its own
/// lock, plus its attachment to a durable medium when it has one.
#[derive(Debug, Default)]
pub(crate) struct DurabilitySubsystem {
    state: RuntimeOwnedState<DurabilityState>,
    checkpoint_selection: RelationalCheckpointManifestSelection,
}

impl DurabilitySubsystem {
    pub(crate) fn has_source_affinity(&self) -> bool {
        {
            let state = self.state.read();
            state.medium.is_some()
                || state.transition_history.is_some()
                || state.cold_source.is_some()
        }
    }
    pub(crate) fn checkpoint_selection(&self) -> &RelationalCheckpointManifestSelection {
        &self.checkpoint_selection
    }

    pub(crate) fn cold_source(
        &self,
    ) -> Option<Arc<dyn crate::durability::medium::RelationalDurableColdSource>> {
        self.state.read().cold_source.clone()
    }

    pub(crate) fn attach_cold_source(
        &mut self,
        source: Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    ) {
        self.state.write().cold_source = Some(source);
    }

    pub(crate) fn attach_medium(&mut self, medium: Box<dyn RelationalDurableMedium>) {
        self.attach_recovered_medium(
            crate::durability::published_medium::RelationalDurableMediumTransfer::fresh(medium),
        );
    }

    pub(crate) fn attach_recovered_medium(
        &mut self,
        transfer: crate::durability::published_medium::RelationalDurableMediumTransfer,
    ) {
        let binding = Arc::new(RelationalDurableMediumBinding::new(transfer));
        let source = binding.cold_source();
        let mut state = self.state.write();
        state.cold_source = Some(source);
        state.medium = Some(binding);
    }

    /// Move the medium out so it can move into the runtime recovery rebuilt.
    /// This runtime, and every port that holds its attachment, then refuses
    /// to publish. `None` when it has no medium or the medium already moved.
    pub(crate) fn detach_medium(
        &self,
    ) -> Option<crate::durability::published_medium::RelationalDurableMediumTransfer> {
        self.medium().as_ref().and_then(|binding| binding.detach())
    }

    pub(crate) fn medium(&self) -> Option<Arc<RelationalDurableMediumBinding>> {
        self.state.read().medium.clone()
    }

    pub(crate) fn transition_history(&self) -> Option<RelationalTransitionHistoryReader> {
        let (medium, reader) = {
            let state = self.state.read();
            (state.medium.clone(), state.transition_history.clone())
        };
        medium.map(|medium| medium.transition_history()).or(reader)
    }

    pub(crate) fn canonical_coverage(
        &self,
    ) -> commit_log::read_coverage::DurableCanonicalCoverage<'_> {
        commit_log::read_coverage::DurableCanonicalCoverage::new(&self.state)
    }

    pub(crate) fn commit_log(&self) -> commit_log::DurabilityCommitLog<'_> {
        commit_log::DurabilityCommitLog::new(&self.state)
    }

    pub(crate) fn push_checkpoint(&self, checkpoint: DurableCheckpoint) {
        self.state.write().push_checkpoint(checkpoint);
    }

    pub(crate) fn set_log(
        &self,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        log: Vec<(
            CommitId,
            crate::publication::patch::data::PatchStreamPosition,
            Arc<crate::history::RelationalCommitEnvelopeResidency>,
        )>,
    ) {
        self.state.write().set_log(lane, log);
    }

    #[cfg(test)]
    pub(crate) fn remove_log_commit(&self, commit_id: CommitId) -> bool {
        self.state.write().remove_log_commit(commit_id)
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
        self.canonical_coverage().log()
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
        self.canonical_coverage().checkpoints()
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
            checkpoint_selection: Default::default(),
        }
    }

    fn fork(&self) -> Self {
        let reader = self.transition_history();
        Self {
            state: RuntimeOwnedState::new(self.state.read().fork_snapshot(reader)),
            checkpoint_selection: self.checkpoint_selection.detached(),
        }
    }
}

mod service_alias_retirement;
