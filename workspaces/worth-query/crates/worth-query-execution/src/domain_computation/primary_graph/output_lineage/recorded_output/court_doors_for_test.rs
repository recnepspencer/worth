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
    pub fn completed_decision_reuse_for_test(
        &self,
    ) -> &Option<super::super::CompletedDecisionReuseProof> {
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
    pub fn prepared_input_reuse_key_for_test(
        &self,
    ) -> &Option<super::super::PreparedInputReuseKey> {
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
    pub fn completed_decision_reuse_mut_for_test(
        &mut self,
    ) -> &mut Option<super::super::CompletedDecisionReuseProof> {
        &mut self.completed_decision_reuse
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn completed_handler_facts_mut_for_test(
        &mut self,
    ) -> &mut Option<CompletedHandlerFactBoundary> {
        &mut self.completed_handler_facts
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn prepared_input_reuse_key_mut_for_test(
        &mut self,
    ) -> &mut Option<super::super::PreparedInputReuseKey> {
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
    pub fn native_output_witness_for_test(
        &self,
    ) -> &OnceLock<Arc<OnceLock<super::super::SealedNativeOutputWitness>>> {
        &self.native_output_witness
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn native_prior_checkpoint_for_test(
        &self,
    ) -> &Option<super::super::native_prior_checkpoint::NativePriorCheckpointLocator> {
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
    pub fn producing_implementation_for_test(
        &self,
    ) -> &Option<worth_relational::facade::mvcc::RequiredOutputProducingImplementation> {
        &self.producing_implementation
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn required_publication_for_test(
        &self,
    ) -> &Option<worth_relational::facade::mvcc::RelationalRequiredOutputPublication> {
        &self.required_publication
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn performed_origin_for_test(&self) -> &Option<Arc<OnceLock<RecordedOutput>>> {
        &self.performed_origin
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn consumed_outputs_for_test(&self) -> &Arc<[ConsumedOutputEvidence]> {
        &self.consumed_outputs
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn idempotency_key_identity_for_test(&self) -> &[u8; 32] {
        &self.idempotency_key_identity
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn mutable_for_test(&self) -> &Mutex<RecordedOutputMutable> {
        &self.mutable
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn source_identity_for_test(&self) -> &Option<RecordedSourceIdentity> {
        &self.source_identity
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn native_prior_checkpoint_mut_for_test(
        &mut self,
    ) -> &mut Option<super::super::native_prior_checkpoint::NativePriorCheckpointLocator> {
        &mut self.native_prior_checkpoint
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn performed_origin_mut_for_test(&mut self) -> &mut Option<Arc<OnceLock<RecordedOutput>>> {
        &mut self.performed_origin
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn checkpoint_source_facts_for_test(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload>
    {
        Self::checkpoint_source_facts(self)
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn native_output_witness_cell_for_test(
        &self,
    ) -> Option<&Arc<OnceLock<super::super::SealedNativeOutputWitness>>> {
        Self::native_output_witness_cell(self)
    }
}

impl RecordedOutputMutable {
    #[doc(hidden)]
    pub fn new_for_test(
        verification_requirement: Option<FullVerificationReason>,
        facts: Option<super::super::RetainedSourceFacts>,
        computation: super::super::retained_computation::RecordedComputation,
    ) -> Self {
        Self::new(verification_requirement, facts, computation)
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn require_verification_for_test(&self, reason: FullVerificationReason) {
        Self::require_verification(self, reason)
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn restore_for_test(
        &self,
        facts: crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
        verified_witness: Option<Arc<OnceLock<super::super::SealedNativeOutputWitness>>>,
    ) {
        Self::restore(self, facts, verified_witness)
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn verification_requirement_for_test(&self) -> Option<FullVerificationReason> {
        Self::verification_requirement(self)
    }
}
