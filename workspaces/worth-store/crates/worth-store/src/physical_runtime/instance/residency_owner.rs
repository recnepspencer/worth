use crate::physical_runtime::record_serving::RecordFramePorts;

mod recovery_admission;
use recovery_admission::RecoveryAllocationBinding;

/// Sole lifecycle owner for one Store instance's physical residency pool.
///
/// `RecordFramePorts` clones are capability facades for serving consumers.
/// They cannot replace this non-cloneable owner or suppress its terminal close.
pub(in crate::physical_runtime) struct PhysicalResidencyOwner {
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    admitted_policy: crate::physical_runtime::record_serving::AdmittedPhysicalRecordResidencyPolicy,
    recovery_allocation: RecoveryAllocationBinding,
    ports: RecordFramePorts,
    closed: bool,
}

impl PhysicalResidencyOwner {
    pub(in crate::physical_runtime) fn admit(
        store: worth_store_physical_format::store_namespace::StableStoreIdentity,
        admitted_policy:
            crate::physical_runtime::record_serving::AdmittedPhysicalRecordResidencyPolicy,
    ) -> Result<Self, worth_store_buffer_pool::PhysicalResidencyDenial> {
        let recovery_allocation = crate::physical_runtime::PhysicalRecoveryAllocationAdmission::new(
            store,
            admitted_policy
                .scope_bytes(worth_store_buffer_pool::PhysicalOperationAllocationScope::Recovery),
        );
        Ok(Self {
            store,
            admitted_policy,
            recovery_allocation: RecoveryAllocationBinding::ServingPolicy(recovery_allocation),
            ports: RecordFramePorts::bounded(store, admitted_policy.limits())?,
            closed: false,
        })
    }

    pub(in crate::physical_runtime) const fn ports(&self) -> &RecordFramePorts {
        &self.ports
    }

    pub(in crate::physical_runtime) const fn admitted_policy(
        &self,
    ) -> crate::physical_runtime::record_serving::AdmittedPhysicalRecordResidencyPolicy {
        self.admitted_policy
    }

    pub(in crate::physical_runtime) fn validate_recovered_policy(
        &self,
        store: worth_store_physical_format::store_namespace::StableStoreIdentity,
        policy: crate::physical_runtime::record_serving::AdmittedPhysicalRecordResidencyPolicy,
    ) -> Result<(), crate::physical_runtime::record_serving::RecordBootstrapDenial> {
        if self.store != store {
            return Err(crate::physical_runtime::record_serving::RecordBootstrapDenial::RecoveredResidencyStoreMismatch);
        }
        if self.admitted_policy != policy {
            return Err(crate::physical_runtime::record_serving::RecordBootstrapDenial::RecoveredResidencyPolicyMismatch);
        }
        Ok(())
    }

    pub(in crate::physical_runtime) fn observation(
        &self,
        generation: crate::physical_runtime::LifecycleGeneration,
    ) -> crate::physical_runtime::record_serving::PhysicalResidencyObservation {
        crate::physical_runtime::record_serving::PhysicalResidencyObservation::new(
            self.store,
            generation,
            self.admitted_policy,
            self.ports.counters(),
            self.ports.allocation_events().snapshot(),
            self.ports.writeback_counters(),
        )
    }

    pub(in crate::physical_runtime) fn close(
        mut self,
    ) -> worth_store_buffer_pool::PhysicalResidencyShutdown {
        let shutdown = self.ports.close();
        self.closed = true;
        shutdown
    }
}

impl Drop for PhysicalResidencyOwner {
    fn drop(&mut self) {
        if !self.closed {
            let _shutdown = self.ports.close();
            self.closed = true;
        }
    }
}

#[cfg(test)]
mod tests;
