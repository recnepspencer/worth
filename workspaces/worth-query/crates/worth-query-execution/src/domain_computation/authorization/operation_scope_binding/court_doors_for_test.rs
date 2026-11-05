//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding;


impl WorthQueryOperationScopeBinding {
    #[doc(hidden)]
    pub fn axis_probe_scope_for_test(
        runtime_authority: u64,
        binding_identity: ApplicationSchemaBindingIdentity,
        operation_authority_identity: &str,
        principal_partition: u32,
        principal_slot: u64,
        principal_generation: u32,
        scope_partition: u32,
        scope_slot: u64,
        scope_generation: u32,
    ) -> Self {
        Self::axis_probe_scope(
            runtime_authority,
            binding_identity,
            operation_authority_identity,
            principal_partition,
            principal_slot,
            principal_generation,
            scope_partition,
            scope_slot,
            scope_generation,
        )
    }
}
