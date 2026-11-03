//! Sealed test doors over this module's private members.
use super::*;

impl RecordedOutput {
    #[doc(hidden)]
    pub fn settlement_identity_for_test(&self) -> &Arc<RecordedSettlementIdentity> {
        &self.settlement_identity
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn completed_decision_reuse_for_test(&self) -> &Option<super::super::CompletedDecisionReuseProof> {
        &self.completed_decision_reuse
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn completed_handler_facts_for_test(&self) -> &Option<CompletedHandlerFactBoundary> {
        &self.completed_handler_facts
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn prepared_input_reuse_key_for_test(&self) -> &Option<super::super::PreparedInputReuseKey> {
        &self.prepared_input_reuse_key
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn source_partition_identity_for_test(&self) -> &Option<[u8; 32]> {
        &self.source_partition_identity
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn completed_decision_reuse_mut_for_test(&mut self) -> &mut Option<super::super::CompletedDecisionReuseProof> {
        &mut self.completed_decision_reuse
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn completed_handler_facts_mut_for_test(&mut self) -> &mut Option<CompletedHandlerFactBoundary> {
        &mut self.completed_handler_facts
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn prepared_input_reuse_key_mut_for_test(&mut self) -> &mut Option<super::super::PreparedInputReuseKey> {
        &mut self.prepared_input_reuse_key
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn observed_source_facts_for_test(&self) -> Option<super::super::RetainedSourceFacts> {
        Self::observed_source_facts(self)
    }
}
