//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn retained_output_candidates_metered_for_test(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
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
