use std::collections::HashMap;
use std::sync::Arc;

use crate::durability::data::DurableCheckpoint;
use crate::history::data::{CommitId, PositionedCanonicalCommit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CheckpointEnvelopeLocation {
    checkpoint_index: usize,
    envelope_index: usize,
}

/// The durability subsystem's authoritative contents.
///
/// Envelopes and checkpoints are held by shared ownership so a reader can
/// carry them out of the subsystem lock without copying them.
#[derive(Debug, Clone, Default)]
pub(crate) struct DurabilityState {
    pub(super) log: Vec<Arc<PositionedCanonicalCommit>>,
    pub(super) checkpoints: Vec<Arc<DurableCheckpoint>>,
    log_commit_index: HashMap<CommitId, usize>,
    checkpoint_commit_index: HashMap<CommitId, CheckpointEnvelopeLocation>,
}

impl DurabilityState {
    pub(super) fn push_log_envelope(&mut self, commit: PositionedCanonicalCommit) {
        let commit_id = commit.envelope().commit.commit_id;
        self.log_commit_index.insert(commit_id, self.log.len());
        self.log.push(Arc::new(commit));
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

    pub(super) fn set_log(&mut self, log: Vec<PositionedCanonicalCommit>) {
        self.log = log.into_iter().map(Arc::new).collect();
        self.rebuild_log_commit_index();
    }

    pub(super) fn rebuild_log_commit_index(&mut self) {
        self.log_commit_index.clear();
        for (index, envelope) in self.log.iter().enumerate() {
            self.log_commit_index
                .insert(envelope.envelope().commit.commit_id, index);
        }
    }

    #[cfg(test)]
    pub(super) fn remove_log_commit(&mut self, commit_id: CommitId) -> bool {
        let before = self.log.len();
        self.log
            .retain(|entry| entry.envelope().commit.commit_id != commit_id);
        let changed = before != self.log.len();
        if changed {
            self.rebuild_log_commit_index();
        }
        changed
    }

    /// Drop every envelope at or below a checkpoint's coverage frontier.
    pub(super) fn retain_log_after(&mut self, commit_id: CommitId) {
        self.log.retain(|entry| entry.commit.commit_id > commit_id);
        self.rebuild_log_commit_index();
    }

    pub(super) fn trim_log_front(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        self.log.drain(0..count);
        self.rebuild_log_commit_index();
    }

    pub(super) fn durable_log_envelope(
        &self,
        commit_id: CommitId,
    ) -> Option<Arc<PositionedCanonicalCommit>> {
        self.log_commit_index
            .get(&commit_id)
            .and_then(|index| self.log.get(*index))
            .map(Arc::clone)
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
