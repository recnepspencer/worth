//! Sealed test doors over this module's private members.
use super::*;

impl PreparedOutputLineageSlot {
    #[doc(hidden)]
    pub fn computation_for_test(&self) -> &PreparedComputationCustody {
        &self.computation
    }
}

impl PreparedOutputLineageSlot {
    #[doc(hidden)]
    pub fn retain_computation_for_test(
        &mut self,
        sealed: SealedComputationRetention,
        prior: Option<super::super::PriorComputationRecord>,
    ) {
        Self::retain_computation(self, sealed, prior)
    }
}

impl CancelledLineageSlot {
    #[doc(hidden)]
    pub fn new_for_test(
        identity: Arc<RecordedSettlementIdentity>,
        partition: Option<[u8; 32]>,
        next: Option<Box<Self>>,
    ) -> Self {
        Self {
            identity,
            partition,
            next,
        }
    }
}

impl PreparedOutputLineageSlot {
    #[doc(hidden)]
    pub fn new_for_test(
        owner: Arc<Mutex<WorthQueryApplicationOutputLineage>>,
        output_binding_type: TypeId,
        source: SemanticSource,
        coordinate: ProductCoordinate,
        partition: Option<[u8; 32]>,
        identity: Arc<RecordedSettlementIdentity>,
        record_cell: Arc<OnceLock<RecordedOutput>>,
        partition_cell: Arc<OnceLock<usize>>,
        cancellation: Option<Box<CancelledLineageSlot>>,
        completed_handler_facts: Option<
        crate::domain_computation::primary_graph::application_attempt::CompletedHandlerFactBoundary,
    >,
        completed_decision_reuse: Option<super::super::CompletedDecisionReuseProof>,
        prepared_input_reuse_key: Option<super::super::PreparedInputReuseKey>,
        native_output_witness: Option<Arc<OnceLock<super::super::SealedNativeOutputWitness>>>,
        required_publication: Option<
            worth_relational::facade::mvcc::RelationalRequiredOutputPublication,
        >,
        native_prior_checkpoint: Option<
            super::super::native_prior_checkpoint::NativePriorCheckpointLocator,
        >,
        prior_computation: Option<super::super::PriorComputationRecord>,
        filled: bool,
        computation: PreparedComputationCustody,
        coupled_completion: Option<super::super::CoupledOutputCompletion>,
    ) -> Self {
        Self {
            owner,
            output_binding_type,
            source,
            coordinate,
            partition,
            identity,
            record_cell,
            partition_cell,
            cancellation,
            completed_handler_facts,
            completed_decision_reuse,
            prepared_input_reuse_key,
            native_output_witness,
            required_publication,
            native_prior_checkpoint,
            prior_computation,
            filled,
            computation,
            coupled_completion,
        }
    }
}
