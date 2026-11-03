//! Read-only canonical lineage event projection.
use super::*;

impl CanonicalCommitEnvelope {
    pub fn lineage_event_ids(&self) -> &[u64] {
        self.lineage.lineage_event_ids()
    }
}

impl CanonicalCommitEnvelope {
    pub fn lineage_events(&self) -> &[LineageEventRecord] {
        self.lineage.lineage_events()
    }
    pub fn lineage_decision_log(&self) -> &[LineageDecisionRecord] {
        self.lineage.lineage_decision_log()
    }
    pub fn lineage_decisions_for_event_id(&self, event_id: u64) -> Vec<&LineageDecisionRecord> {
        self.lineage.decisions_for_event_id(event_id).collect()
    }

    pub fn lineage_digest_basis(&self) -> &LineageDigestBasis {
        self.lineage.digest_basis()
    }
    pub fn event_batch_digest_basis(&self) -> &LineageEventBatchDigestBasis {
        self.lineage.event_batch_digest_basis()
    }

    pub fn decision_log_digest_basis(&self) -> &LineageDecisionLogDigestBasis {
        self.lineage.decision_log_digest_basis()
    }

    pub fn lineage_artifact_counters(&self) -> LineageArtifactCounters {
        self.lineage.counters()
    }

    pub fn has_lineage_authority(&self) -> bool {
        self.lineage.has_authority_content()
    }
}
