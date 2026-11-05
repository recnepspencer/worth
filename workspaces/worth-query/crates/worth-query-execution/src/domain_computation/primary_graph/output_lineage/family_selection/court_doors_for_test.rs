//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn resolve_current_family_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding,
        family: &str,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
    ) -> Result<WorthQueryCurrentOutputFamilyResolution, CompanionPreflightStop> {
        Self::resolve_current_family(
            self,
            execution,
            runtime_authority,
            schema,
            scope,
            family,
            occurrence,
            generation,
        )
    }
}
