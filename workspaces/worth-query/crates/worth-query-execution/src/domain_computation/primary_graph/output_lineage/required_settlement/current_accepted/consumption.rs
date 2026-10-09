//! A bound accepted row supplies the complete basis of retained consumption.
use super::*;

impl BoundCurrentAcceptedOutput<'_> {
    pub(in crate::domain_computation::primary_graph) fn retain_consumed(
        self,
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<ConsumedOutputEvidence, ConsumedOutputVerificationStop> {
        ConsumedOutputEvidence::retain_bound(self, owner, admission)
    }

    pub(in crate::domain_computation::primary_graph) fn consumed_identity(
        &self,
    ) -> &std::sync::Arc<super::super::super::RecordedSettlementIdentity> {
        &self
            ._proof
            .candidate
            .selected
            .recorded()
            .settlement_identity
    }
    pub(in crate::domain_computation::primary_graph) fn consumed_facts(
        &self,
    ) -> &super::super::super::ComparableSourceFacts {
        &self._proof.facts
    }
    pub(in crate::domain_computation::primary_graph) fn consumed_upstream(
        &self,
    ) -> &std::sync::Arc<[ConsumedOutputEvidence]> {
        &self._proof.candidate.selected.recorded().consumed_outputs
    }
    pub(in crate::domain_computation::primary_graph) fn consumed_witness(
        &self,
    ) -> Option<&std::sync::Arc<std::sync::OnceLock<super::super::super::SealedNativeOutputWitness>>>
    {
        self._proof.candidate.selected.native_output_witness_cell()
    }
    pub(in crate::domain_computation::primary_graph) fn consumed_root(
        &self,
    ) -> &PositionedRelationalSnapshot {
        self._proof._selected
    }
}
