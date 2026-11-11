//! Existing accounting effects separated from their prepared recorder evidence.
use super::PhysicalResidencyAccounting;
use crate::{PhysicalOperationAllocationScope, PhysicalResidencyRetirementDenial as Denial};

impl PhysicalResidencyAccounting {
    pub(crate) fn retirement_recorder(
        &self,
    ) -> super::super::PhysicalResidencyAllocationEventRecorder {
        self.events.clone()
    }

    pub(crate) fn check_pin_retirement(&self, last_pin: bool) -> Result<(), Denial> {
        if self.counters.pin_leases == 0 || (last_pin && self.counters.pinned_frames == 0) {
            return Err(Denial::AccountingMismatch);
        }
        Ok(())
    }

    pub(crate) fn release_pin_counters(&mut self, last_pin: bool) {
        self.counters.pin_leases -= 1;
        if last_pin {
            self.counters.pinned_frames -= 1;
        }
    }

    pub(crate) fn check_operation_retirement(
        &self,
        scope: PhysicalOperationAllocationScope,
        bytes: u64,
    ) -> Result<(), Denial> {
        if self.counters.active_operation_bytes < bytes
            || self.counters.operation_scope_bytes[scope.index()] < bytes
        {
            return Err(Denial::AccountingMismatch);
        }
        Ok(())
    }

    pub(crate) fn release_operation_counters(
        &mut self,
        scope: PhysicalOperationAllocationScope,
        bytes: u64,
    ) {
        self.counters.active_operation_bytes -= bytes;
        self.counters.operation_scope_bytes[scope.index()] -= bytes;
    }
}
