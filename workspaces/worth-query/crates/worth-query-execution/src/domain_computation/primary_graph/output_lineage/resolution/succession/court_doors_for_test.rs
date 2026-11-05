//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn resolve_binding_succession_for_test<Preserve: 'static, Create: 'static>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        partition: [u8; 32],
    ) -> Result<WorthQueryPriorOutputBindingResolution, Stop> {
        Self::resolve_binding_succession::<Preserve, Create>(
            self, execution, scope, occurrence, generation, partition,
        )
    }
}
