//! Rebuild one envelope’s lineage cache for the in-crate court.
use super::CanonicalCommitEnvelopeBody;

impl CanonicalCommitEnvelopeBody {
    pub(crate) fn rebuild_lineage_basis_for_test(&mut self) {
        let events = crate::lineage::data::FinalizedLineageEventBatch::new(
            self.lineage.lineage_events().to_vec(),
        );
        let decisions = crate::lineage::data::LineageDecisionLog::new(
            self.lineage.lineage_decision_log().to_vec(),
        );
        self.lineage = crate::lineage::data::LineageFinalizationArtifact::new(
            self.branch_context.clone(),
            events,
            decisions,
        )
        .publish();
    }
}
