//! Install and trim the exact performed envelope log under the durable policy.
use super::state::DurabilityState;
use std::sync::Arc;
use worth_relational_branch::internal_port::history::commit::envelope_residency::RelationalCommitEnvelopeResidency;
use worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use worth_relational_contracts::internal_port::config::data::policies::DurableLogPolicy;
use worth_relational_storage::internal_port::storage::substrate::owned_state::RuntimeOwnedState;

pub struct DurabilityCommitLog<'a> {
    state: &'a RuntimeOwnedState<DurabilityState>,
}

impl<'a> DurabilityCommitLog<'a> {
    pub fn new(state: &'a RuntimeOwnedState<DurabilityState>) -> Self {
        Self { state }
    }

    pub fn append(
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

    pub fn compact_if_needed(&self, policy: &DurableLogPolicy) {
        use worth_relational_contracts::internal_port::config::data::policies::DurableLogRetentionMode;

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

pub mod read_coverage;
