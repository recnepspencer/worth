//! Sealed test doors over this module's private members.
use super::*;

impl<'a> NativePriorCheckpointOutput<'a> {
    #[doc(hidden)]
    pub fn identity_for_test(&self) -> &Option<NativePriorCheckpointIdentity<'a>> {
        &self.identity
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn checkpoint_prior_outputs_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        occurrence: ProductBranchIncarnation,
        generation: u64,
    ) -> Result<Vec<NativePriorCheckpointOutput<'_>>, Denial> {
        Self::checkpoint_prior_outputs(
            self,
            execution,
            runtime_authority,
            schema,
            occurrence,
            generation,
        )
    }
}
