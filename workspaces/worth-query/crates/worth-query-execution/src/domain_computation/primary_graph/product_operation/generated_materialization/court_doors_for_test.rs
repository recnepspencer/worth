//! Sealed test doors over this module's private members.
use super::*;

impl ProducerQualification {
    #[doc(hidden)]
    pub fn new_for_test(
        binding_type: TypeId,
        output_binding_type: TypeId,
        binding_identity: &'static str,
        provider_identity: &'static str,
        runtime_source_identity: crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryRuntimeSourceIdentity,
        checkpoint_source_identity: crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryCheckpointSourceIdentity,
        recorded_source_identity: crate::domain_computation::primary_graph::output_lineage::RecordedSourceIdentity,
        source_partition_identity: [u8; 32],
        producer_dependency_identity: Option<[u8; 32]>,
        producing_implementation: Option<
            worth_relational::facade::mvcc::RequiredOutputProducingImplementation,
        >,
        idempotency_key_identity: [u8; 32],
        runtime_authority: u64,
        schema: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding,
        output_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        output_generation: u64,
        observed_source_facts: crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts,
    ) -> Self {
        Self {
            binding_type,
            output_binding_type,
            binding_identity,
            provider_identity,
            runtime_source_identity,
            checkpoint_source_identity,
            recorded_source_identity,
            source_partition_identity,
            producer_dependency_identity,
            producing_implementation,
            idempotency_key_identity,
            runtime_authority,
            schema,
            scope,
            output_occurrence,
            output_generation,
            observed_source_facts,
        }
    }
}
