//! Original Recovery admission binding in the single native residency owner.

use crate::physical_runtime::{LifecycleGeneration, PhysicalRecoveryAllocationAdmission};
use worth_store_buffer_pool::{PhysicalOperationAllocationScope as Scope, PhysicalResidencyDenial};

use super::PhysicalResidencyOwner;

pub(super) enum RecoveryAllocationBinding {
    ServingPolicy(PhysicalRecoveryAllocationAdmission),
    RecoveryAdmission {
        admission: PhysicalRecoveryAllocationAdmission,
        origin_generation: LifecycleGeneration,
    },
}

impl PhysicalResidencyOwner {
    /// Bind the carried original ceiling before exposing any Recovery issuer.
    /// Rebinding cannot widen it, even when the target policy permits more.
    pub(in crate::physical_runtime) fn restrict_recovery_allocation(
        &mut self,
        original: PhysicalRecoveryAllocationAdmission,
        origin_generation: LifecycleGeneration,
    ) -> Result<(), PhysicalResidencyDenial> {
        if original.store_identity() != self.store {
            return Err(PhysicalResidencyDenial::WrongStore);
        }
        if let RecoveryAllocationBinding::RecoveryAdmission {
            origin_generation: prior,
            ..
        } = self.recovery_allocation
        {
            if prior != origin_generation {
                return Err(PhysicalResidencyDenial::AllocationGrantMismatch);
            }
        }
        let retained = match self.recovery_allocation {
            RecoveryAllocationBinding::RecoveryAdmission {
                admission: prior, ..
            } if prior.byte_limit() < original.byte_limit() => prior,
            _ => original,
        };
        self.ports
            .restrict_recovery_operation_bytes(retained.byte_limit())?;
        // Native denial must leave the carried diagnostic basis unchanged.
        self.recovery_allocation = RecoveryAllocationBinding::RecoveryAdmission {
            admission: retained,
            origin_generation,
        };
        Ok(())
    }

    pub(in crate::physical_runtime) const fn recovery_allocation_admission(
        &self,
    ) -> PhysicalRecoveryAllocationAdmission {
        match self.recovery_allocation {
            RecoveryAllocationBinding::ServingPolicy(admission)
            | RecoveryAllocationBinding::RecoveryAdmission { admission, .. } => admission,
        }
    }

    pub(in crate::physical_runtime) const fn recovery_origin_generation(
        &self,
    ) -> Option<LifecycleGeneration> {
        match self.recovery_allocation {
            RecoveryAllocationBinding::ServingPolicy(_) => None,
            RecoveryAllocationBinding::RecoveryAdmission {
                origin_generation, ..
            } => Some(origin_generation),
        }
    }

    /// Observational sizing hint, not allocation permission. Native admission
    /// atomically checks all live charges again before issuing or growing a grant.
    pub(in crate::physical_runtime) fn available_recovery_operation_bytes(&self) -> u64 {
        let scope = Scope::Recovery;
        let limits = self.admitted_policy.limits();
        let counters = self.ports.counters();
        [
            limits
                .usable_bytes(scope, limits.scope_bytes(scope))
                .saturating_sub(counters.active_operation_bytes_for(scope)),
            limits
                .usable_bytes(scope, limits.operation_bytes())
                .saturating_sub(counters.active_operation_bytes()),
            limits
                .usable_bytes(scope, limits.total_bytes())
                .saturating_sub(counters.admitted_bytes()),
            self.recovery_allocation_admission()
                .byte_limit()
                .saturating_sub(counters.active_operation_bytes_for(scope)),
        ]
        .into_iter()
        .min()
        .expect("Recovery sizing has four admitted dimensions")
    }
}
