//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn admit_performed_for_test(
        &self,
        requested_key: WorthQueryOutputDemandKey,
        source_commit: &worth_runtime_world::facade::CompositeCommitIdentity,
        source_scope: crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding,
        product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> Result<WorthQueryOutputDemandInterest, WorthQueryOutputDemandDenial> {
        Self::admit_performed(
            self,
            requested_key,
            source_commit,
            source_scope,
            product_occurrence,
        )
    }
}
