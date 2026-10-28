use super::PerformanceAccess;
use crate::replay::data::{ReplayAuthorityBasisKind, ReplayVerificationLayer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReplayLineageAuthorityIndexedSource {
    DurableLog,
    Checkpoint,
}

impl PerformanceAccess<'_> {
    pub(crate) fn count_replay_verification_layer(&self, layer: ReplayVerificationLayer) {
        self.counters.count(|counters| match layer {
            ReplayVerificationLayer::DigestParity => counters.replay_digest_parity_checks += 1,
            ReplayVerificationLayer::SummaryParity => counters.replay_summary_parity_checks += 1,
            ReplayVerificationLayer::DeepArtifactParity => {
                counters.replay_deep_artifact_parity_checks += 1
            }
        });
    }

    pub(crate) fn count_replay_lineage_authority_basis(
        &self,
        indexed_source: Option<ReplayLineageAuthorityIndexedSource>,
        kind: ReplayAuthorityBasisKind,
        event_width: usize,
        decision_width: usize,
    ) {
        self.counters.count(|counters| {
            counters.replay_lineage_authority_lookup_requests += 1;
            match indexed_source {
                Some(ReplayLineageAuthorityIndexedSource::DurableLog) => {
                    counters.replay_lineage_log_index_hits += 1;
                }
                Some(ReplayLineageAuthorityIndexedSource::Checkpoint) => {
                    counters.replay_lineage_checkpoint_index_hits += 1;
                }
                None => {}
            }
            match kind {
                ReplayAuthorityBasisKind::DurableLogCanonical => {
                    counters.replay_lineage_durable_basis_selections += 1;
                }
                ReplayAuthorityBasisKind::RetainedEnvelopeCanonical => {
                    counters.replay_lineage_retained_envelope_basis_selections += 1;
                }
            }
            counters.replay_lineage_digest_event_width += event_width;
            counters.replay_lineage_digest_decision_width += decision_width;
        });
    }

    pub(crate) fn count_replay_lineage_authoritative_basis_rejection(&self) {
        self.counters.count(|counters| {
            counters.replay_lineage_authoritative_basis_rejections += 1;
        });
    }
}
