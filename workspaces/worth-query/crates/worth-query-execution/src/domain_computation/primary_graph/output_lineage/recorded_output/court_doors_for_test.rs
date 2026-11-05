//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::primary_graph::handler_fact_boundary::CompletedHandlerFactBoundary;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::mark_state::FullVerificationReason;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;

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
    ) -> &Option<crate::domain_computation::vocabulary::primary_graph::decision_reuse::CompletedDecisionReuseProof> {
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
    ) -> &mut Option<crate::domain_computation::vocabulary::primary_graph::decision_reuse::CompletedDecisionReuseProof> {
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

impl RecordedOutputMutable {
    #[doc(hidden)]
    pub fn computation_for_test(&self) -> &super::super::retained_computation::RecordedComputation {
        &self.computation
    }
}

impl RecordedOutputMutable {
    #[doc(hidden)]
    pub fn computation_mut_for_test(
        &mut self,
    ) -> &mut super::super::retained_computation::RecordedComputation {
        &mut self.computation
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn correspondence_mut_for_test(
        &mut self,
    ) -> &mut Arc<WorthQueryApplicationOutputCorrespondence> {
        &mut self.correspondence
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn producing_implementation_mut_for_test(
        &mut self,
    ) -> &mut Option<worth_relational::facade::mvcc::RequiredOutputProducingImplementation> {
        &mut self.producing_implementation
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn settlement_identity_mut_for_test(&mut self) -> &mut Arc<RecordedSettlementIdentity> {
        &mut self.settlement_identity
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn source_partition_identity_mut_for_test(&mut self) -> &mut Option<[u8; 32]> {
        &mut self.source_partition_identity
    }
}

impl RecordedOutputMutable {
    #[doc(hidden)]
    pub fn set_stable_fact_spans_for_test(
        &mut self,
        spans: super::super::input_cutoff::stable_publication::StableFactSpans,
    ) {
        Self::set_stable_fact_spans(self, spans)
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn mutable_mut_for_test(&mut self) -> &mut Mutex<RecordedOutputMutable> {
        &mut self.mutable
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn computation_source_mut_for_test(
        &mut self,
    ) -> &mut super::super::ComputationSourceEvidence {
        &mut self.computation_source
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn consumed_outputs_mut_for_test(&mut self) -> &mut Arc<[ConsumedOutputEvidence]> {
        &mut self.consumed_outputs
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn native_output_witness_mut_for_test(
        &mut self,
    ) -> &mut OnceLock<Arc<OnceLock<super::super::SealedNativeOutputWitness>>> {
        &mut self.native_output_witness
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn required_publication_mut_for_test(
        &mut self,
    ) -> &mut Option<worth_relational::facade::mvcc::RelationalRequiredOutputPublication> {
        &mut self.required_publication
    }
}

impl RecordedOutput {
    #[doc(hidden)]
    pub fn new_for_test(
        producing_implementation: Option<
            worth_relational::facade::mvcc::RequiredOutputProducingImplementation,
        >,
        required_publication: Option<
            worth_relational::facade::mvcc::RelationalRequiredOutputPublication,
        >,
        computation_source: super::super::ComputationSourceEvidence,
        native_prior_checkpoint: Option<
            super::super::native_prior_checkpoint::NativePriorCheckpointLocator,
        >,
        performed_origin: Option<Arc<OnceLock<RecordedOutput>>>,
        consumed_outputs: Arc<[ConsumedOutputEvidence]>,
        completed_handler_facts: Option<CompletedHandlerFactBoundary>,
        completed_decision_reuse: Option<crate::domain_computation::vocabulary::primary_graph::decision_reuse::CompletedDecisionReuseProof>,
        prepared_input_reuse_key: Option<super::super::PreparedInputReuseKey>,
        native_output_witness: OnceLock<Arc<OnceLock<super::super::SealedNativeOutputWitness>>>,
        settlement_identity: Arc<RecordedSettlementIdentity>,
        correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
        source_identity: Option<RecordedSourceIdentity>,
        source_partition_identity: Option<[u8; 32]>,
        producer_dependency_identity: Option<[u8; 32]>,
        idempotency_key_identity: [u8; 32],
        mutable: Mutex<RecordedOutputMutable>,
    ) -> Self {
        Self {
            producing_implementation,
            required_publication,
            computation_source,
            native_prior_checkpoint,
            performed_origin,
            consumed_outputs,
            completed_handler_facts,
            completed_decision_reuse,
            prepared_input_reuse_key,
            native_output_witness,
            settlement_identity,
            correspondence,
            source_identity,
            source_partition_identity,
            producer_dependency_identity,
            idempotency_key_identity,
            mutable,
        }
    }
}
