//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn source_facts_for_receipt_for_test(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        receipt: &WorthQueryApplicationCommitReceipt,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<RetainedOutputCurrentnessRead>, CompanionPreflightStop> {
        Self::source_facts_for_receipt(
            self,
            runtime_authority,
            schema,
            observation,
            receipt,
            execution,
        )
    }
}

impl RetainedOutputCurrentnessRead {
    #[doc(hidden)]
    pub fn facts_for_test(&self) -> &Option<super::super::ComparableSourceFacts> {
        &self.facts
    }
}

impl RetainedOutputCurrentnessRead {
    #[doc(hidden)]
    pub fn verification_requirement_for_test(
        &self,
    ) -> &Option<crate::domain_computation::primary_graph::vocabulary::output_lineage::mark_state::FullVerificationReason>{
        &self.verification_requirement
    }
}

impl RetainedOutputCurrentnessRead {
    #[doc(hidden)]
    pub fn verification_requirement_mut_for_test(
        &mut self,
    ) -> &mut Option<crate::domain_computation::primary_graph::vocabulary::output_lineage::mark_state::FullVerificationReason>{
        &mut self.verification_requirement
    }
}

impl RetainedOutputCurrentnessRead {
    #[doc(hidden)]
    pub fn facts_mut_for_test(&mut self) -> &mut Option<super::super::ComparableSourceFacts> {
        &mut self.facts
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn current_output_matches_receipt_for_test(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        receipt: &WorthQueryApplicationCommitReceipt,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<bool, CompanionPreflightStop> {
        Self::current_output_matches_receipt(
            self,
            runtime_authority,
            schema,
            observation,
            receipt,
            execution,
        )
    }
}

impl RetainedOutputCurrentnessRead {
    #[doc(hidden)]
    pub fn new_for_test(
        identity: Arc<crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity>,
        facts: Option<super::super::ComparableSourceFacts>,
        consumed_outputs: Arc<[ConsumedOutputEvidence]>,
        native_output_witness: Option<Arc<OnceLock<super::super::SealedNativeOutputWitness>>>,
        consumed_nothing: bool,
        verification_requirement: Option<crate::domain_computation::primary_graph::vocabulary::output_lineage::mark_state::FullVerificationReason>,
    ) -> Self {
        Self {
            identity,
            facts,
            consumed_outputs,
            native_output_witness,
            consumed_nothing,
            verification_requirement,
        }
    }
}
