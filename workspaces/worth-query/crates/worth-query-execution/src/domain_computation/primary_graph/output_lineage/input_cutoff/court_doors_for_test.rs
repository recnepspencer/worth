//! Sealed test doors over this module's private members.
use super::*;

impl RetainedInputCutoffCandidate {
    #[doc(hidden)]
    pub fn from_exact_cell_for_test(cell: Arc<OnceLock<RecordedOutput>>) -> Self {
        Self::from_exact_cell(cell)
    }
}

impl RetainedInputCutoffCandidate {
    #[doc(hidden)]
    pub fn native_output_witness_for_test(&self) -> Option<&SealedNativeOutputWitness> {
        Self::native_output_witness(self)
    }
}

impl RetainedInputCutoffCandidate {
    #[doc(hidden)]
    pub fn observed_source_facts_for_test(
        &self,
    ) -> Result<Option<super::super::ComparableSourceFacts>, CompanionPreflightStop> {
        Self::observed_source_facts(self)
    }
}

impl RetainedInputCutoffCandidate {
    #[doc(hidden)]
    pub fn prepared_input_key_for_test(&self) -> Option<&PreparedInputReuseKey> {
        Self::prepared_input_key(self)
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn prior_input_cutoff_candidate_for_test<Binding: 'static>(
        &self,
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
        observation: &ProductBranchObservation,
        source_partition_identity: [u8; 32],
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<RetainedInputCutoffCandidate>, CompanionPreflightStop> {
        Self::prior_input_cutoff_candidate::<Binding>(
            self,
            scope,
            observation,
            source_partition_identity,
            execution,
        )
    }
}

impl RetainedInputCutoffCandidate {
    #[doc(hidden)]
    pub fn settlement_identity_for_test(&self) -> &Arc<RecordedSettlementIdentity> {
        Self::settlement_identity(self)
    }
}
