use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::durability::data::DurableCheckpoint;
use crate::history::data::{CommitId, PositionedCanonicalCommit};
use crate::history::RelationalCommitEnvelopeResidency;
use crate::publication::patch::data::PatchStreamPosition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CheckpointEnvelopeLocation {
    checkpoint_index: usize,
    envelope_index: usize,
}

/// The durability subsystem's authoritative contents.
///
/// Envelopes and checkpoints are held by shared ownership so a reader can
/// carry them out of the subsystem lock without copying them.
#[derive(Debug, Default)]
pub(crate) struct DurabilityState {
    pub(super) medium: Option<Arc<crate::durability::medium::RelationalDurableMediumBinding>>,
    pub(super) transition_history:
        Option<crate::durability::medium::RelationalTransitionHistoryReader>,
    pub(super) cold_source: Option<Arc<dyn crate::durability::medium::RelationalDurableColdSource>>,
    pub(super) log: VecDeque<(
        CommitId,
        PatchStreamPosition,
        Arc<RelationalCommitEnvelopeResidency>,
    )>,
    pub(super) checkpoints: Vec<Arc<DurableCheckpoint>>,
    pub(super) source_roots: Option<Arc<crate::durability::RecoveredRootInventory>>,
    log_commit_index: HashMap<CommitId, Arc<RelationalCommitEnvelopeResidency>>,
    checkpoint_commit_index: HashMap<CommitId, CheckpointEnvelopeLocation>,
}

impl DurabilityState {
    pub(super) fn fork_snapshot(
        &self,
        transition_history: Option<crate::durability::medium::RelationalTransitionHistoryReader>,
    ) -> Self {
        Self {
            medium: None,
            transition_history,
            cold_source: self.cold_source.clone(),
            log: self.log.clone(),
            checkpoints: self.checkpoints.clone(),
            // A fork has its own actual branch inventory. It does not acquire
            // the source owner's separate checkpoint-image admissions.
            source_roots: None,
            log_commit_index: self.log_commit_index.clone(),
            checkpoint_commit_index: self.checkpoint_commit_index.clone(),
        }
    }

    pub(super) fn push_log_envelope(
        &mut self,
        commit: (
            CommitId,
            PatchStreamPosition,
            Arc<RelationalCommitEnvelopeResidency>,
        ),
    ) {
        self.log_commit_index.insert(commit.0, commit.2.clone());
        self.log.push_back(commit);
    }

    pub(super) fn push_checkpoint(&mut self, checkpoint: DurableCheckpoint) {
        // Encoding runs outside publication locks, so captures can complete
        // out of order. Never replace the basis for an already compacted tail
        // with an older coverage frontier. Equal coverage can carry updated
        // derived index images and remains a valid new checkpoint.
        if self.checkpoints.last().is_some_and(|current| {
            crate::durability::access::checkpoint_coverage(current)
                >= crate::durability::access::checkpoint_coverage(&checkpoint)
                && current.coverage.up_to_branch_transition
                    >= checkpoint.coverage.up_to_branch_transition
                && (crate::durability::access::checkpoint_coverage(current)
                    > crate::durability::access::checkpoint_coverage(&checkpoint)
                    || current.coverage.up_to_branch_transition
                        > checkpoint.coverage.up_to_branch_transition)
        }) {
            return;
        }
        let checkpoint_index = self.checkpoints.len();
        for (envelope_index, envelope) in checkpoint.envelopes.iter().enumerate() {
            self.checkpoint_commit_index.insert(
                envelope.envelope().commit.commit_id,
                CheckpointEnvelopeLocation {
                    checkpoint_index,
                    envelope_index,
                },
            );
        }
        self.checkpoints.push(Arc::new(checkpoint));
    }

    pub(super) fn set_log(
        &mut self,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        log: Vec<(
            CommitId,
            PatchStreamPosition,
            Arc<RelationalCommitEnvelopeResidency>,
        )>,
    ) {
        self.log = log.into();
        self.rebuild_log_commit_index(lane);
    }

    pub(super) fn rebuild_log_commit_index(
        &mut self,
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    ) {
        self.log_commit_index.clear();
        for envelope in &self.log {
            self.log_commit_index.insert(envelope.0, envelope.2.clone());
        }
    }

    #[cfg(test)]
    pub(super) fn remove_log_commit(&mut self, commit_id: CommitId) -> bool {
        let before = self.log.len();
        self.log.retain(|entry| entry.0 != commit_id);
        let changed = before != self.log.len();
        if changed {
            self.rebuild_log_commit_index(
                &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
            );
        }
        changed
    }

    /// Drop every envelope at or below a checkpoint's coverage frontier.
    pub(super) fn retain_log_after(&mut self, commit_id: CommitId) {
        self.log.retain(|entry| {
            let retain = entry.0 > commit_id;
            if !retain {
                self.log_commit_index.remove(&entry.0);
            }
            retain
        });
    }

    pub(super) fn trim_log_front(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        for entry in self.log.drain(0..count) {
            self.log_commit_index.remove(&entry.0);
        }
    }

    pub(super) fn durable_log_envelope(
        &self,
        commit_id: CommitId,
    ) -> Option<Arc<PositionedCanonicalCommit>> {
        self.log_commit_index
            .get(&commit_id)
            .and_then(|entry| entry.positioned_if_resident())
            .map(Arc::new)
    }

    pub(super) fn checkpoint_envelope(
        &self,
        commit_id: CommitId,
    ) -> Option<PositionedCanonicalCommit> {
        let location = self.checkpoint_commit_index.get(&commit_id)?;
        self.checkpoints
            .get(location.checkpoint_index)
            .and_then(|checkpoint| checkpoint.envelopes.get(location.envelope_index))
            .cloned()
    }
}
