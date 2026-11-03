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

impl RecordedOutput {
    #[doc(hidden)]
    pub fn correspondence_for_test(&self) -> &Arc<WorthQueryApplicationOutputCorrespondence> {
        &self.correspondence
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn native_output_witness_for_test(&self) -> &OnceLock<Arc<OnceLock<super::super::SealedNativeOutputWitness>>> {
        &self.native_output_witness
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn native_prior_checkpoint_for_test(&self) -> &Option<super::super::native_prior_checkpoint::NativePriorCheckpointLocator> {
        &self.native_prior_checkpoint
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn producer_dependency_identity_for_test(&self) -> &Option<[u8; 32]> {
        &self.producer_dependency_identity
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn producing_implementation_for_test(&self) -> &Option<worth_relational::facade::mvcc::RequiredOutputProducingImplementation> {
        &self.producing_implementation
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn required_publication_for_test(&self) -> &Option<worth_relational::facade::mvcc::RelationalRequiredOutputPublication> {
        &self.required_publication
    }
}
