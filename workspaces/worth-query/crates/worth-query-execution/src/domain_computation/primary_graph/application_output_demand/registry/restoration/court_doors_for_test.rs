//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRestoredAcceptedOutput {
    #[doc(hidden)]
    pub fn checkpoint_for_test(&self) -> &super::super::WorthQueryAcceptedOutputCheckpointIdentity {
        match &self.origin {
            RestoredOutputOrigin::Checkpoint(checkpoint) => checkpoint,
            _ => panic!("checkpoint court requires checkpoint origin"),
        }
    }
}

impl WorthQueryRestoredAcceptedOutput {
    #[doc(hidden)]
    pub fn source_scope_for_test(
        &self,
    ) -> &crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding {
        &self.source_scope
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn admit_restored_for_test(
        &self,
        requested_key: WorthQueryOutputDemandKey,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        admission_kind: super::super::DemandAdmissionKind,
        restored: super::super::WorthQueryRestoredAcceptedOutput,
        settlement: &std::sync::Arc<RecordedSettlementIdentity>,
        performed_source: Option<&worth_runtime_world::facade::CompositeCommitIdentity>,
    ) -> Result<(WorthQueryOutputDemandInterest, bool), WorthQueryOutputDemandDenial> {
        Self::admit_restored(
            self,
            requested_key,
            source_scope,
            product_occurrence,
            admission_kind,
            restored,
            settlement,
            performed_source,
        )
    }
}

impl WorthQueryRestoredAcceptedOutput {
    #[doc(hidden)]
    pub fn checkpoint_mut_for_test(
        &mut self,
    ) -> &mut super::super::WorthQueryAcceptedOutputCheckpointIdentity {
        match &mut self.origin {
            RestoredOutputOrigin::Checkpoint(checkpoint) => checkpoint,
            _ => panic!("checkpoint court requires checkpoint origin"),
        }
    }
}

impl WorthQueryRestoredAcceptedOutput {
    #[doc(hidden)]
    pub fn new_for_test(
        checkpoint: super::super::WorthQueryAcceptedOutputCheckpointIdentity,
        correspondence: std::sync::Arc<
            crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
        >,
        observation: worth_runtime_world::facade::ProductBranchObservation,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        source_identity: crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity,
        observed_source_facts: crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
        native_output_witness: Option<
        std::sync::Arc<std::sync::OnceLock<crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness>>,
    >,
    ) -> Self {
        Self {
            origin: RestoredOutputOrigin::Checkpoint(checkpoint),
            correspondence,
            observation,
            source_scope,
            source_identity,
            observed_source_facts,
            native_output_witness,
        }
    }
}
