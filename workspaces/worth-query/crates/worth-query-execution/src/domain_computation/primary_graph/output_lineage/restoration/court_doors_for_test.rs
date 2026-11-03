//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub fn record_restoration_for_test(
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
        observed_source_facts: super::super::RetainedSourceFacts,
        verified_witness: Option<Arc<OnceLock<super::super::SealedNativeOutputWitness>>>,
        checkpoint: Option<crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity>,
        producing_implementation: Option<worth_relational::facade::mvcc::RequiredOutputProducingImplementation>,
    ) -> Option<RestoredRecord> {
        Self::record_restoration(
            self, output_binding, runtime_authority, schema, scope, observation, correspondence,
            source_identity, source_partition_identity, producer_dependency_identity,
            idempotency_key_identity, observed_source_facts, verified_witness, checkpoint,
            producing_implementation,
        )
    }
}
