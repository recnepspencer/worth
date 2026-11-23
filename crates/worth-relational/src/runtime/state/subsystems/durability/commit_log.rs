//! Install and trim the exact performed envelope log under the durable policy.
use super::state::DurabilityState;
use crate::config::data::DurableLogPolicy;
use crate::history::commit::RelationalCommitEnvelopeResidency;
use crate::history::data::PositionedCanonicalCommit;
use crate::storage::substrate::owned_state::RuntimeOwnedState;
use std::sync::Arc;

pub(crate) struct DurabilityCommitLog<'a> {
    state: &'a RuntimeOwnedState<DurabilityState>,
}

impl<'a> DurabilityCommitLog<'a> {
    pub(super) fn new(state: &'a RuntimeOwnedState<DurabilityState>) -> Self {
        Self { state }
    }

    pub(crate) fn append(
        &self,
        positioned: &PositionedCanonicalCommit,
        cell: Arc<RelationalCommitEnvelopeResidency>,
    ) {
        self.state.write().push_log_envelope((
            positioned.commit.commit_id,
            positioned.position(),
            cell,
        ));
    }

    pub(crate) fn compact_if_needed(&self, policy: &DurableLogPolicy) {
        use crate::config::data::DurableLogRetentionMode;

        if self.state.read().log.len() <= policy.max_in_memory_envelopes {
            return;
        }

        match policy.retention_mode {
            DurableLogRetentionMode::RetainAllInMemory => {}
            DurableLogRetentionMode::CompactAfterCheckpoint => {
                let checkpoint = self.state.read().checkpoints.last().cloned();
                if let Some(checkpoint) = checkpoint
                    && let Some(commit) = checkpoint.coverage.up_to_commit.as_ref()
                {
                    self.state.write().retain_log_after(commit.commit_id);
                }
                let log_len = self.state.read().log.len();
                if log_len > policy.max_in_memory_envelopes {
                    self.state
                        .write()
                        .trim_log_front(log_len - policy.max_in_memory_envelopes);
                }
            }
        }
    }
}
