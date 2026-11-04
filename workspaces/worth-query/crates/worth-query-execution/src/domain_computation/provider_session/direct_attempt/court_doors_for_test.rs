//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryDirectExecutionResourceAttempt {
    #[doc(hidden)]
    pub fn start_for_test(
        reserved: WorthQueryCapacityReservedExecutionResourcePlan,
        binding_authority: &crate::domain_computation::operation_binding::WorthQueryExecutionBoundOperationAuthority,
    ) -> Self {
        Self::start(reserved, binding_authority)
    }
}

impl WorthQueryDirectExecutionResourceAttempt {
    #[doc(hidden)]
    pub fn binding_authority_for_test(
        &self,
    ) -> &crate::domain_computation::operation_binding::WorthQueryExecutionBoundOperationAuthority
    {
        Self::binding_authority(self)
    }
}
