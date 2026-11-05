//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn record_prior_output_for_test(
        &mut self,
        output_binding: TypeId,
        runtime_authority: u64,
        schema: ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
        origin: PriorOutputOrigin,
        source_partition_identity: [u8; 32],
        producer_dependency_identity: Option<[u8; 32]>,
        idempotency_key_identity: [u8; 32],
    ) {
        Self::record_prior_output(
            self,
            output_binding,
            runtime_authority,
            schema,
            scope,
            occurrence,
            generation,
            correspondence,
            origin,
            source_partition_identity,
            producer_dependency_identity,
            idempotency_key_identity,
        )
    }
}
