use std::{fmt, marker::PhantomData};

use worth_store_buffer_pool::OperationAllocationGrant;
use worth_store_physical_format::store_namespace::StableStoreIdentity;

use crate::physical_runtime::{
    record_serving::residency::frame_ports::RecordFramePorts, LifecycleGeneration, RuntimeIdentity,
};

use super::scope;

pub(super) struct StoreScopedAllocation<'runtime, Scope> {
    grant: OperationAllocationGrant,
    _runtime_owner: &'runtime RecordFramePorts,
    runtime: RuntimeIdentity,
    generation: LifecycleGeneration,
    _scope: PhantomData<fn() -> Scope>,
}

impl<Scope> fmt::Debug for StoreScopedAllocation<'_, Scope> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StoreScopedAllocation")
            .field("grant", &self.grant)
            .field("runtime", &self.runtime)
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}

impl<'runtime, Scope> StoreScopedAllocation<'runtime, Scope> {
    pub(super) fn from_pool_grant(
        grant: OperationAllocationGrant,
        runtime_owner: &'runtime RecordFramePorts,
        runtime: RuntimeIdentity,
        generation: LifecycleGeneration,
    ) -> Self {
        Self {
            grant,
            _runtime_owner: runtime_owner,
            runtime,
            generation,
            _scope: PhantomData,
        }
    }
}

macro_rules! exact_scope_allocation {
    ($name:ident, $scope:ident) => {
        #[derive(Debug)]
        /// Runtime-bound temporary-byte ownership for one exact physical scope.
        ///
        /// Dropping this value releases its Store operation charge. It carries
        /// Store and runtime identity for downstream binding but grants no pool,
        /// scheduler, effect, or successor-policy authority.
        pub struct $name<'runtime> {
            allocation: StoreScopedAllocation<'runtime, scope::$scope>,
        }

        impl<'runtime> $name<'runtime> {
            pub(super) fn bind(allocation: StoreScopedAllocation<'runtime, scope::$scope>) -> Self {
                Self { allocation }
            }

            /// Returns the stable Store whose envelope owns this allocation.
            pub fn store_identity(&self) -> StableStoreIdentity {
                self.allocation.grant.observation().store()
            }

            /// Returns the serving generation that admitted this allocation.
            pub const fn store_generation(&self) -> LifecycleGeneration {
                self.allocation.generation
            }

            /// Returns the exact runtime that must remain live with this allocation.
            pub const fn runtime_identity(&self) -> RuntimeIdentity {
                self.allocation.runtime
            }

            /// Returns the exact charged byte count.
            pub const fn bytes(&self) -> u64 {
                self.allocation.grant.bytes()
            }
        }
    };
}

exact_scope_allocation!(RecoveryPhysicalAllocation, RecoveryScope);
exact_scope_allocation!(ScrubPhysicalAllocation, ScrubScope);
exact_scope_allocation!(MaintenancePhysicalAllocation, MaintenanceScope);
exact_scope_allocation!(VerificationPhysicalAllocation, VerificationScope);
exact_scope_allocation!(BlobPhysicalAllocation, BlobScope);
exact_scope_allocation!(LayoutPhysicalAllocation, LayoutReadScope);

/// The maintenance charge for retired directory records after preparation
/// stops borrowing the Serving frame ports. Its pool lease remains owned by
/// every prepared mutation clone that can still read those records.
#[derive(Debug)]
pub(in crate::physical_runtime) struct MaintenanceRetainedDirectoryCharge {
    grant: OperationAllocationGrant,
    runtime: RuntimeIdentity,
    generation: LifecycleGeneration,
}

impl MaintenancePhysicalAllocation<'_> {
    pub(in crate::physical_runtime) fn try_resize(
        &mut self,
        bytes: u64,
    ) -> Result<(), super::PhysicalScopedAllocationFailure> {
        self.allocation.grant.try_resize(bytes).map_err(|denial| {
            super::PhysicalScopedAllocationFailure::from_denial(denial, self.allocation.generation)
        })
    }

    pub(in crate::physical_runtime) fn into_retained_directory_charge(
        self,
    ) -> MaintenanceRetainedDirectoryCharge {
        MaintenanceRetainedDirectoryCharge {
            grant: self.allocation.grant,
            runtime: self.allocation.runtime,
            generation: self.allocation.generation,
        }
    }
}

impl MaintenanceRetainedDirectoryCharge {
    pub(in crate::physical_runtime) fn matches(
        &self,
        store: StableStoreIdentity,
        runtime: RuntimeIdentity,
        generation: LifecycleGeneration,
    ) -> bool {
        self.grant.observation().store() == store
            && self.runtime == runtime
            && self.generation == generation
    }
}

impl BlobPhysicalAllocation<'_> {
    pub(in crate::physical_runtime::record_serving) fn operation_grant(
        &self,
    ) -> &OperationAllocationGrant {
        &self.allocation.grant
    }
}
