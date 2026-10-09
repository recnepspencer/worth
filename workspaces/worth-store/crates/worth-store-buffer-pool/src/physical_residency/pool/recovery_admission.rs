//! Monotonic Recovery operation capacity restricted under native admission's lock.

use super::{
    PhysicalOperationAllocationScope, PhysicalResidencyDenial, PhysicalResidencyDimension,
    PhysicalResidencyPressureDemand, PoolInner,
};

impl PoolInner {
    pub(super) fn restrict_recovery_operation_bytes(
        &self,
        ceiling: u64,
    ) -> Result<(), PhysicalResidencyDenial> {
        let mut state = self.lock();
        if !state.accepting {
            return Err(Self::deny(&mut state, PhysicalResidencyDenial::PoolClosed));
        }
        let scope = PhysicalOperationAllocationScope::Recovery;
        let current = state.accounting.operation_scope_bytes(scope);
        if ceiling < current {
            return Err(self.pressure(
                &mut state,
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::OperationScope(scope),
                    scope,
                    requested: 0,
                    current,
                    limit: ceiling,
                },
            ));
        }
        state.recovery_operation_bytes_ceiling =
            state.recovery_operation_bytes_ceiling.min(ceiling);
        Ok(())
    }
}
