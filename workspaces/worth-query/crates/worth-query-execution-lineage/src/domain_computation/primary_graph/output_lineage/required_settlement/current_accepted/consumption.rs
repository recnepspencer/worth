//! A bound accepted row supplies the complete basis of retained consumption.
use super::*;

impl BoundCurrentAcceptedOutput<'_> {
    pub fn retain_consumed(self) -> Result<ConsumedOutputEvidence, ConsumedOutputVerificationStop> {
        ConsumedOutputEvidence::retain_bound(self)
    }

    pub fn consumed_identity(
        &self,
    ) -> &std::sync::Arc<super::super::super::RecordedSettlementIdentity> {
        &self
            ._proof
            .candidate
            .selected
            .recorded()
            .settlement_identity
    }
    pub fn consumed_facts(&self) -> &super::super::super::ComparableSourceFacts {
        &self._proof.facts
    }
    /// The upstream this row consumed, as its root reads it (public #358).
    pub fn consumed_upstream(&self) -> std::sync::Arc<[ConsumedOutputEvidence]> {
        std::sync::Arc::clone(&self._proof
            .candidate
            .selected
            .recorded().consumed_outputs)
    }
    pub fn consumed_witness(
        &self,
    ) -> Option<&std::sync::Arc<std::sync::OnceLock<super::super::super::SealedNativeOutputWitness>>>
    {
        self._proof.candidate.selected.native_output_witness_cell()
    }
    pub fn consumed_root(&self) -> &PositionedRelationalSnapshot {
        self._proof._selected
    }
}
