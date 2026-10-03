//! Native-backed temporary vectors and qualified artifact-tree reads for one
//! completed-history walk. The final raw-slice grant moves to Serving.

use worth_store_buffer_pool::OperationAllocationGrant;
use worth_store_physical_backend::{
    ArtifactTreePathAllocationBoundary, ArtifactTreePathAllocator, ArtifactTreeReadAllocator,
    ArtifactTreeStorageAllocator, ObservedRecoveryArtifact, RecoveryDiscoveryAllocationFailure,
};
use worth_store_physical_format::RecordArtifactFile;
use worth_store_physical_format::{
    DurableExtentRecordPlacement, EXTENT_ARENA_MANIFEST_FRAME_BYTES,
};

use super::super::{
    control_frames::FundedCompletedHistoricalRawSlices,
    control_frames::{read_extent_with_storage, ExtentReadStorage, SelectedArtifactSlice},
    resident::StoreRejoinResidentLedger,
    tier::{no_release_controls::FailedIngestFrameStorage, routes::RouteWalkStorage},
    SelectedMediaRejoinDenial as Denial,
};
use crate::physical_runtime::{
    PhysicalRecoveryObservationAllocationDenial as ReadDenial, PhysicalRecoveryReadAllocation,
    PhysicalRecoveryRejoinResidentDenial as ResidentDenial,
};

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) struct HistoricalWalkStorage<
    'scope,
    'owner,
> {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) window:
        &'scope PhysicalRecoveryReadAllocation<'owner>,
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) resident:
        &'scope mut StoreRejoinResidentLedger,
    backing: &'scope mut FundedCompletedHistoricalRawSlices,
}

impl<'scope, 'owner> HistoricalWalkStorage<'scope, 'owner> {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn new(
        window: &'scope PhysicalRecoveryReadAllocation<'owner>,
        resident: &'scope mut StoreRejoinResidentLedger,
        backing: &'scope mut FundedCompletedHistoricalRawSlices,
    ) -> Self {
        Self {
            window,
            resident,
            backing,
        }
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn reserve_vec<T>(
        &mut self,
        count: usize,
    ) -> Result<Vec<T>, Denial> {
        self.reserve_storage(count).map_err(|denial| match denial {
            ReadDenial::Residency(cause) => Denial::Resident(cause),
            ReadDenial::AllocatorExceededReservation { requested, actual } => {
                Denial::Resident(ResidentDenial::AllocatorExceededReservation { requested, actual })
            }
            _ => Denial::BoundExceeded,
        })
    }

    fn reserve_storage<T>(&mut self, count: usize) -> Result<Vec<T>, ReadDenial> {
        let admitted = self.window.recovery_byte_limit();
        let overflow = || ReadDenial::Residency(ResidentDenial::SizeOverflow { admitted });
        let requested = slot_bytes::<T>(count).ok_or_else(overflow)?;
        self.resident
            .transient(requested)
            .map_err(ReadDenial::Residency)?;
        let next = self
            .backing
            .retained_bytes()
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
            ReadDenial::Residency(ResidentDenial::Allocation { requested, cause })
        })?;
        let actual = slot_bytes::<T>(values.capacity()).ok_or_else(overflow)?;
        if actual > requested {
            return Err(ReadDenial::AllocatorExceededReservation { requested, actual });
        }
        self.resident
            .retain(actual)
            .map_err(ReadDenial::Residency)?;
        self.backing.retain(actual).map_err(|_| overflow())?;
        self.backing
            .settle(self.backing.retained_bytes())
            .map_err(|_| overflow())?;
        Ok(values)
    }

    /// A replacement vector is funded while the old vector remains live.
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn grow_vec<T>(
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
        let mut fresh = self.reserve_vec(needed.max(values.capacity().saturating_mul(2)))?;
        fresh.append(values);
        let old = std::mem::replace(values, fresh);
        self.discard_vec(old)
    }

    /// Moves a vector funded only by the resident ledger into the raw grant
    /// that travels to Serving. Both copies stay charged until the original
    /// is dropped.
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn adopt_resident_vec<
        T: Clone,
    >(
        &mut self,
        values: Vec<T>,
    ) -> Result<Vec<T>, Denial> {
        let bytes = self
            .resident
            .vector_bytes(&values)
            .map_err(Denial::Resident)?;
        let mut funded = self.reserve_vec(values.len())?;
        funded.extend_from_slice(&values);
        drop(values);
        self.resident.release(bytes);
        Ok(funded)
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn discard_vec<T>(
        &mut self,
        values: Vec<T>,
    ) -> Result<(), Denial> {
        let bytes = slot_bytes::<T>(values.capacity()).ok_or(Denial::BoundExceeded)?;
        drop(values);
        self.resident.release(bytes);
        self.release_raw_bytes(bytes)
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn release_raw_bytes(
        &mut self,
        bytes: u64,
    ) -> Result<(), Denial> {
        let next = self
            .backing
            .retained_bytes()
            .checked_sub(bytes)
            .ok_or(Denial::BoundExceeded)?;
        self.backing.settle(next)
    }
}

impl ArtifactTreeStorageAllocator for HistoricalWalkStorage<'_, '_> {
    type Denial = ReadDenial;
}

impl ArtifactTreePathAllocator for HistoricalWalkStorage<'_, '_> {
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

impl ArtifactTreeReadAllocator for HistoricalWalkStorage<'_, '_> {
    fn allocate_read_buffer(&mut self, requested: usize) -> Result<Vec<u8>, ReadDenial> {
        let mut bytes = self.reserve_storage(requested)?;
        bytes.resize(requested, 0);
        Ok(bytes)
    }
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn read_denial(
    failure: RecoveryDiscoveryAllocationFailure<ReadDenial>,
) -> Denial {
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

fn slot_bytes<T>(count: usize) -> Option<u64> {
    count
        .checked_mul(std::mem::size_of::<T>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .and_then(|bytes| u64::try_from(bytes).ok())
}

impl RouteWalkStorage for HistoricalWalkStorage<'_, '_> {
    fn reserve_vec<T>(&mut self, count: usize) -> Result<Vec<T>, Denial> {
        HistoricalWalkStorage::reserve_vec(self, count)
    }
    fn grow_vec<T>(&mut self, values: &mut Vec<T>, additional: usize) -> Result<(), Denial> {
        HistoricalWalkStorage::grow_vec(self, values, additional)
    }
    fn grow_vec_geometrically<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), Denial> {
        HistoricalWalkStorage::grow_vec(self, values, additional)
    }
    fn discard_vec<T>(&mut self, values: Vec<T>) -> Result<(), Denial> {
        HistoricalWalkStorage::discard_vec(self, values)
    }
    fn release_consumed_bytes(&mut self, bytes: u64) -> Result<(), Denial> {
        self.resident.release(bytes);
        self.release_raw_bytes(bytes)
    }
    fn read_page(
        &mut self,
        discovery: &mut worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery,
        artifact: RecordArtifactFile,
        limit: u64,
    ) -> Result<ObservedRecoveryArtifact, Denial> {
        discovery
            .read_record_artifact_with_storage(artifact, limit, self)
            .map_err(read_denial)
    }
    fn read_page_range(
        &mut self,
        discovery: &mut worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery,
        artifact: RecordArtifactFile,
        offset: u64,
        length: u32,
    ) -> Result<ObservedRecoveryArtifact, Denial> {
        discovery
            .read_record_artifact_range_with_storage(
                artifact,
                offset,
                length,
                u64::from(length),
                self,
            )
            .map_err(read_denial)
    }
    fn discard_frame(&mut self, frame: ObservedRecoveryArtifact) -> Result<(), Denial> {
        let bytes = frame.owned_heap_bytes().ok_or_else(|| self.overflow())?;
        drop(frame);
        self.resident.release(bytes);
        self.release_raw_bytes(bytes)
    }
    fn overflow(&self) -> Denial {
        Denial::Resident(ResidentDenial::SizeOverflow {
            admitted: self
                .resident
                .used()
                .saturating_add(self.resident.remaining()),
        })
    }
}

impl ExtentReadStorage for HistoricalWalkStorage<'_, '_> {
    fn grow_slices(&mut self, slices: &mut Vec<SelectedArtifactSlice>) -> Result<(), Denial> {
        self.grow_vec(slices, 1)
    }
    fn reserve_payload(&mut self, count: usize) -> Result<Vec<u8>, Denial> {
        self.reserve_vec(count)
    }
    fn read_manifest(
        &mut self,
        discovery: &mut worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery,
        placement: DurableExtentRecordPlacement,
        page_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, Denial> {
        self.read_extent_range(
            discovery,
            placement,
            0,
            EXTENT_ARENA_MANIFEST_FRAME_BYTES as u32,
            page_limit,
        )
    }
    fn read_chunk(
        &mut self,
        discovery: &mut worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery,
        placement: DurableExtentRecordPlacement,
        relative: u64,
        length: u32,
        page_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, Denial> {
        self.read_extent_range(discovery, placement, relative, length, page_limit)
    }
    fn discard_frame(&mut self, frame: ObservedRecoveryArtifact) -> Result<(), Denial> {
        RouteWalkStorage::discard_frame(self, frame)
    }
}

impl HistoricalWalkStorage<'_, '_> {
    fn read_extent_range(
        &mut self,
        discovery: &mut worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery,
        placement: DurableExtentRecordPlacement,
        relative: u64,
        length: u32,
        page_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, Denial> {
        let range = placement.arena_range();
        let end = relative
            .checked_add(u64::from(length))
            .ok_or(Denial::BoundExceeded)?;
        if end > range.length() {
            return Err(Denial::ControlFrame);
        }
        let offset = range
            .offset()
            .checked_add(relative)
            .ok_or(Denial::BoundExceeded)?;
        discovery
            .read_record_artifact_range_with_storage(
                RecordArtifactFile::ExtentArena {
                    arena: range.arena().get(),
                },
                offset,
                length,
                page_limit,
                self,
            )
            .map_err(read_denial)
    }
}

impl FailedIngestFrameStorage for HistoricalWalkStorage<'_, '_> {
    fn read_selected(
        &mut self,
        discovery: &mut worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery,
        format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
        route: worth_store_physical_format::CurrentPhysicalRecordPlacement,
        maximum: u64,
        slices: &mut Vec<SelectedArtifactSlice>,
        max_slices: usize,
    ) -> Result<Vec<u8>, Denial> {
        let worth_store_physical_format::CurrentPhysicalRecordPlacement::Extent(extent) = route
        else {
            return Err(Denial::UnsupportedSelectedPlacement);
        };
        if extent.payload_bytes() == 0 || extent.payload_bytes() > maximum {
            return Err(Denial::BoundExceeded);
        }
        let (bytes, _) = read_extent_with_storage(discovery, format, extent, slices, self)?;
        if slices.len() > max_slices {
            return Err(Denial::BoundExceeded);
        }
        Ok(bytes)
    }
}
