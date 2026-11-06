//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn qualified_output_for_test<Binding: 'static>(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        runtime_identity: crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity,
        checkpoint_identity: crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity,
    ) -> Option<WorthQueryExactRecordedOutput> {
        Self::qualified_output::<Binding>(
            self,
            runtime_authority,
            schema,
            scope,
            occurrence,
            generation,
            runtime_identity,
            checkpoint_identity,
        )
    }
}
