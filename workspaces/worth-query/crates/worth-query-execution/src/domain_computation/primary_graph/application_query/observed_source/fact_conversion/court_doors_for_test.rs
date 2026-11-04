//! Sealed test doors over this module's private members.
use super::*;

impl<Query> WorthQueryObservedSource<Query> {
    #[doc(hidden)]
    pub fn retained_checkpoint_facts_for_test(
        &self,
        layout: &WorthQueryPrimaryGraphLayout,
        request: Option<
            &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        >,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<std::sync::Arc<[Fact]>, WorthQuerySourceExpectationDenial> {
        Self::retained_checkpoint_facts(self, layout, request, allocation_policy)
    }
}

impl<Query> WorthQueryObservedSource<Query> {
    #[doc(hidden)]
    pub fn validate_and_into_facts_for_test(
        self,
        runtime_authority: u64,
        binding: &ApplicationSchemaBindingIdentity,
        branch: &BranchId,
        model_root: EntityId,
        selected_product: &crate::basis::WorthQueryProductBranchReadIdentity,
        expected_query_identifier: &str,
        expected_query_identity: &WorthQueryInstalledApplicationQueryIdentity,
        layout: &WorthQueryPrimaryGraphLayout,
        request: Option<
            &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        >,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<Fact>, WorthQuerySourceExpectationDenial> {
        Self::validate_and_into_facts(
            self,
            runtime_authority,
            binding,
            branch,
            model_root,
            selected_product,
            expected_query_identifier,
            expected_query_identity,
            layout,
            request,
            execution,
        )
    }
}
