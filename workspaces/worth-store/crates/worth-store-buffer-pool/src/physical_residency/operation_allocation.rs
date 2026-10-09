use std::num::NonZeroU64;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::{
    pool::PoolInner, PhysicalOperationAllocationScope, PhysicalResidencyCounters,
    PhysicalResidencyIncarnation,
};

mod foreground_read;
mod foreground_write;
mod maintenance;

pub use foreground_read::ForegroundReadAllocationGrant;
pub use foreground_write::ForegroundWriteAllocationGrant;
pub use maintenance::MaintenanceAllocationGrant;

#[derive(Debug)]
pub struct OperationAllocationGrant {
    pub(crate) owner: Arc<PoolInner>,
    pub(crate) scope: PhysicalOperationAllocationScope,
    pub(crate) bytes: u64,
    pub(crate) active_use_bytes: AtomicU64,
}

#[derive(Debug)]
pub(crate) struct OperationAllocationUse<'grant> {
    grant: &'grant OperationAllocationGrant,
    bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationAllocationObservation {
    store: StableStoreIdentity,
    pool: PhysicalResidencyIncarnation,
    scope: PhysicalOperationAllocationScope,
    bytes: u64,
    counters: PhysicalResidencyCounters,
}

impl OperationAllocationGrant {
    /// Resize only this pool-issued reservation; a failed growth changes no charge.
    pub fn try_resize(&mut self, target_bytes: u64) -> Result<(), super::PhysicalResidencyDenial> {
        if target_bytes == self.bytes {
            return Ok(());
        }
        let active = self.active_use_bytes.load(Ordering::Acquire);
        if target_bytes < active {
            return Err(
                super::PhysicalResidencyDenial::AllocationGrantResizeBelowActiveUse {
                    requested: target_bytes,
                    active,
                },
            );
        }
        if target_bytes < self.bytes {
            self.owner
                .release_operation(self.scope, self.bytes - target_bytes);
        } else {
            let additional = NonZeroU64::new(target_bytes - self.bytes)
                .expect("strict growth has a positive delta");
            self.owner.reserve_operation(self.scope, additional)?;
        }
        self.bytes = target_bytes;
        Ok(())
    }

    pub(crate) fn scope_for(
        &self,
        owner: &Arc<PoolInner>,
    ) -> Result<PhysicalOperationAllocationScope, super::PhysicalResidencyDenial> {
        if !Arc::ptr_eq(owner, &self.owner) {
            return Err(super::PhysicalResidencyDenial::AllocationGrantMismatch);
        }
        Ok(self.scope)
    }

    pub(crate) fn reserve_use(
        &self,
        owner: &Arc<PoolInner>,
        bytes: u64,
    ) -> Result<OperationAllocationUse<'_>, super::PhysicalResidencyDenial> {
        let scope = self.scope_for(owner)?;
        let result =
            self.active_use_bytes
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                    current
                        .checked_add(bytes)
                        .filter(|next| *next <= self.bytes)
                });
        match result {
            Ok(_) => Ok(OperationAllocationUse { grant: self, bytes }),
            Err(current) => Err(self
                .owner
                .deny_operation_grant_use(scope, bytes, current, self.bytes)),
        }
    }

    pub const fn bytes(&self) -> u64 {
        self.bytes
    }

    pub const fn scope(&self) -> PhysicalOperationAllocationScope {
        self.scope
    }

    pub fn observation(&self) -> OperationAllocationObservation {
        OperationAllocationObservation {
            store: self.owner.store_identity(),
            pool: self.owner.incarnation(),
            scope: self.scope,
            bytes: self.bytes,
            counters: self.owner.counters(),
        }
    }
}

impl Drop for OperationAllocationGrant {
    fn drop(&mut self) {
        assert_eq!(
            self.active_use_bytes.load(Ordering::Acquire),
            0,
            "operation allocation grant dropped with a live named allocation use"
        );
        self.owner.release_operation(self.scope, self.bytes);
    }
}

impl OperationAllocationUse<'_> {
    pub(crate) const fn scope(&self) -> PhysicalOperationAllocationScope {
        self.grant.scope
    }
}

impl Drop for OperationAllocationUse<'_> {
    fn drop(&mut self) {
        let previous = self
            .grant
            .active_use_bytes
            .fetch_sub(self.bytes, Ordering::AcqRel);
        assert!(
            previous >= self.bytes,
            "named operation allocation use released more bytes than it owned"
        );
    }
}

impl OperationAllocationObservation {
    pub const fn store(self) -> StableStoreIdentity {
        self.store
    }

    pub const fn pool(self) -> PhysicalResidencyIncarnation {
        self.pool
    }

    pub const fn scope(self) -> PhysicalOperationAllocationScope {
        self.scope
    }

    pub const fn bytes(self) -> u64 {
        self.bytes
    }

    pub const fn counters(self) -> PhysicalResidencyCounters {
        self.counters
    }
}
