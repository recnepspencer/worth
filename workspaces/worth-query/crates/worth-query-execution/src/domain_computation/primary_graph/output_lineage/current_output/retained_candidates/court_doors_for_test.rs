//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn retained_output_candidates_metered_for_test(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        output_bindings: &[TypeId],
        source_partition_identity: [u8; 32],
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<WorthQueryRetainedOutputCandidate>, CompanionPreflightStop> {
        Self::retained_output_candidates_metered(
            self,
            runtime_authority,
            schema,
            scope,
            occurrence,
            generation,
            output_bindings,
            source_partition_identity,
            execution,
        )
    }
}

impl WorthQueryRetainedOutputCandidate {
    #[doc(hidden)]
    pub fn correspondence_for_test(
        &self,
    ) -> &Arc<crate::domain_computation::primary_graph::vocabulary::application_attempt::output_correspondence::WorthQueryApplicationOutputCorrespondence>
    {
        &self.correspondence
    }
}

impl WorthQueryRetainedOutputCandidate {
    #[doc(hidden)]
    pub fn native_output_witness_for_test(
        &self,
    ) -> &Option<
        Arc<
            OnceLock<
                crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness,
            >,
        >,
    > {
        &self.native_output_witness
    }
}

impl WorthQueryRetainedOutputCandidate {
    #[doc(hidden)]
    pub fn observed_source_facts_for_test(
        &self,
    ) -> &Option<super::super::super::ComparableSourceFacts> {
        &self.observed_source_facts
    }
}

impl WorthQueryRetainedOutputCandidate {
    #[doc(hidden)]
    pub fn source_identity_for_test(&self) -> &Option<RecordedSourceIdentity> {
        &self.source_identity
    }
}
