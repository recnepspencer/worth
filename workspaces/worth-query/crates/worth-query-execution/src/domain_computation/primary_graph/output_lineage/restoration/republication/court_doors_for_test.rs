//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn prepare_republication_for_test(
        &self,
        output_binding: TypeId,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        suspended_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        suspended_generation: u64,
        source_partition_identity: [u8; 32],
        correspondence: &Arc<WorthQueryApplicationOutputCorrespondence>,
        suspended_facts: &super::super::super::RetainedSourceFacts,
        witness: Arc<OnceLock<SealedNativeOutputWitness>>,

        control: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl<'_, '_>,
    ) -> Option<RepublishedOutput> {
        Self::prepare_republication(
            self,
            output_binding,
            runtime_authority,
            schema,
            scope,
            suspended_occurrence,
            suspended_generation,
            source_partition_identity,
            correspondence,
            suspended_facts,
            witness,
            control,
        )
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn record_republished_restoration_for_test(
        &mut self,
        output_binding: TypeId,
        runtime_authority: u64,
        schema: ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
        source_identity: RecordedSourceIdentity,
        source_partition_identity: [u8; 32],
        producer_dependency_identity: Option<[u8; 32]>,
        idempotency_key_identity: [u8; 32],
        suspended_facts: super::super::super::RetainedSourceFacts,
        republished: RepublishedOutput,
    ) -> Option<RepublishedRecord> {
        Self::record_republished_restoration(
            self,
            output_binding,
            runtime_authority,
            schema,
            scope,
            observation,
            correspondence,
            source_identity,
            source_partition_identity,
            producer_dependency_identity,
            idempotency_key_identity,
            suspended_facts,
            republished,
        )
    }
}

impl RepublishedRecord {
    #[doc(hidden)]
    pub fn consumed_outputs_for_test(&self) -> &Arc<[ConsumedOutputEvidence]> {
        &self.consumed_outputs
    }
}

impl RepublishedRecord {
    #[doc(hidden)]
    pub fn facts_for_test(&self) -> &super::super::super::RetainedSourceFacts {
        &self.facts
    }
}

impl RepublishedRecord {
    #[doc(hidden)]
    pub fn identity_for_test(&self) -> &Arc<RecordedSettlementIdentity> {
        &self.identity
    }
}

impl RepublishedRecord {
    #[doc(hidden)]
    pub fn predecessor_for_test(&self) -> &Arc<RecordedSettlementIdentity> {
        &self.predecessor
    }
}

impl RepublishedRecord {
    #[doc(hidden)]
    pub fn witness_for_test(&self) -> &Arc<OnceLock<SealedNativeOutputWitness>> {
        &self.witness
    }
}

impl RepublishedRecord {
    #[doc(hidden)]
    pub fn witness_into_for_test(self) -> Arc<OnceLock<SealedNativeOutputWitness>> {
        self.witness
    }
}

impl RepublishedRecord {
    #[doc(hidden)]
    pub fn consumed_outputs_into_for_test(self) -> Arc<[ConsumedOutputEvidence]> {
        self.consumed_outputs
    }
}

impl RepublishedRecord {
    #[doc(hidden)]
    pub fn facts_into_for_test(self) -> super::super::super::RetainedSourceFacts {
        self.facts
    }
}

impl RepublishedRecord {
    #[doc(hidden)]
    pub fn identity_into_for_test(self) -> Arc<RecordedSettlementIdentity> {
        self.identity
    }
}

impl RepublishedRecord {
    #[doc(hidden)]
    pub fn predecessor_into_for_test(self) -> Arc<RecordedSettlementIdentity> {
        self.predecessor
    }
}
