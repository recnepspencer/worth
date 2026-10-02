//! One backed allocation port for walk vectors and qualified record reads.

use super::{
    backing::{slot_bytes, vector_bytes, HeadWalkBacking},
    Denial, StoreRejoinResidentLedger,
};
use crate::physical_runtime::{
    PhysicalRecoveryObservationAllocationDenial as ReadDenial, PhysicalRecoveryReadAllocation,
    PhysicalRecoveryRejoinResidentDenial,
};
use worth_store_buffer_pool::OperationAllocationGrant;
use worth_store_physical_backend::{
    ArtifactTreePathAllocationBoundary, ArtifactTreePathAllocator, ArtifactTreeReadAllocator,
    ArtifactTreeStorageAllocator, RecoveryDiscoveryAllocationFailure,
};

pub(super) struct HeadWalkStorage<'scope, 'owner> {
    pub(super) window: &'scope PhysicalRecoveryReadAllocation<'owner>,
    pub(super) resident: &'scope mut StoreRejoinResidentLedger,
    backing: &'scope mut HeadWalkBacking,
}

impl<'scope, 'owner> HeadWalkStorage<'scope, 'owner> {
    pub(super) fn new(
        window: &'scope PhysicalRecoveryReadAllocation<'owner>,
        resident: &'scope mut StoreRejoinResidentLedger,
        backing: &'scope mut HeadWalkBacking,
    ) -> Self {
        Self {
            window,
            resident,
            backing,
        }
    }

    pub(super) fn reserve_vec<T>(&mut self, count: usize) -> Result<Vec<T>, Denial> {
        self.reserve_storage(count).map_err(|denial| match denial {
            ReadDenial::Residency(cause) => Denial::Resident(cause),
            ReadDenial::AllocatorExceededReservation { requested, actual } => Denial::Resident(
                PhysicalRecoveryRejoinResidentDenial::AllocatorExceededReservation {
                    requested,
                    actual,
                },
            ),
            _ => Denial::BoundExceeded,
        })
    }

    fn reserve_storage<T>(&mut self, count: usize) -> Result<Vec<T>, ReadDenial> {
        let admitted = self.window.recovery_byte_limit();
        let overflow = || {
            ReadDenial::Residency(PhysicalRecoveryRejoinResidentDenial::SizeOverflow { admitted })
        };
        let requested = slot_bytes::<T>(count).map_err(|_| overflow())?;
        self.resident
            .transient(requested)
            .map_err(ReadDenial::Residency)?;
        let next = self
            .backing
            .used()
            .checked_add(requested)
            .ok_or_else(overflow)?;
        self.backing
            .prepare(self.window, next)
            .map_err(|denial| match denial {
                Denial::Resident(cause) => ReadDenial::Residency(cause),
                _ => ReadDenial::StoreMismatch,
            })?;
        let mut values = Vec::new();
        values.try_reserve_exact(count).map_err(|cause| {
            ReadDenial::Residency(PhysicalRecoveryRejoinResidentDenial::Allocation {
                requested,
                cause,
            })
        })?;
        let actual = vector_bytes(&values).map_err(|_| overflow())?;
        if actual > requested {
            return Err(ReadDenial::AllocatorExceededReservation { requested, actual });
        }
        self.resident
            .retain(actual)
            .map_err(ReadDenial::Residency)?;
        self.backing.retain(actual).map_err(|_| overflow())?;
        Ok(values)
    }

    pub(super) fn reserve_bytes(&mut self, count: usize) -> Result<Vec<u8>, Denial> {
        let mut values = self.reserve_vec(count)?;
        values.resize(count, 0);
        Ok(values)
    }

    pub(super) fn grow_vec<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), Denial> {
        let needed = values
            .len()
            .checked_add(additional)
            .ok_or(Denial::BoundExceeded)?;
        if needed <= values.capacity() {
            return Ok(());
        }
        // A separate new allocation leaves the original vector intact on denial.
        let mut next = self.reserve_vec(needed)?;
        next.append(values);
        let old = std::mem::replace(values, next);
        self.discard_vec(old)
    }

    pub(super) fn discard_vec<T>(&mut self, values: Vec<T>) -> Result<(), Denial> {
        let bytes = vector_bytes(&values)?;
        drop(values);
        self.resident.release(bytes);
        self.backing.settle(
            self.backing
                .used()
                .checked_sub(bytes)
                .ok_or(Denial::BoundExceeded)?,
        )
    }
}

impl ArtifactTreeStorageAllocator for HeadWalkStorage<'_, '_> {
    type Denial = ReadDenial;
}
impl ArtifactTreePathAllocator for HeadWalkStorage<'_, '_> {
    type PathBacking = Option<OperationAllocationGrant>;
    fn admit_path_backing(
        &mut self,
        boundary: ArtifactTreePathAllocationBoundary,
        bytes: u64,
    ) -> Result<Self::PathBacking, ReadDenial> {
        self.window
            .reserve_owned(bytes)
            .map_err(|cause| ReadDenial::PathResidency { boundary, cause })
    }
}
impl ArtifactTreeReadAllocator for HeadWalkStorage<'_, '_> {
    fn allocate_read_buffer(&mut self, requested: usize) -> Result<Vec<u8>, ReadDenial> {
        let mut values = self.reserve_storage(requested)?;
        values.resize(requested, 0);
        Ok(values)
    }
}

pub(super) fn read_denial(failure: RecoveryDiscoveryAllocationFailure<ReadDenial>) -> Denial {
    match failure {
        RecoveryDiscoveryAllocationFailure::Discovery(failure) => Denial::Discovery(failure),
        RecoveryDiscoveryAllocationFailure::Allocation {
            artifact,
            offset,
            requested,
            cause,
        } => Denial::RecordReadAllocation {
            artifact,
            offset,
            requested,
            cause,
        },
        RecoveryDiscoveryAllocationFailure::BufferLengthMismatch {
            artifact,
            offset,
            requested,
            observed,
        } => Denial::ReadBufferLengthMismatch {
            artifact,
            offset,
            requested,
            observed,
        },
    }
}
