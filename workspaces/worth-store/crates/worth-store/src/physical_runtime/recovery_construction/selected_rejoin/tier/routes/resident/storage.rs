//! Route walk allocation port. The route and topology predicates do not know
//! whether their caller uses the carried resident ledger or completed native
//! backing; both observe the same verified page and entry sequence.

use worth_store_physical_backend::{
    ArtifactCeiling, BoundedRecoveryFilesystemDiscovery, ObservedRecoveryArtifact, PageAddress,
    ReadGrant, UnchargedRead,
};
use worth_store_physical_format::{PhysicalRecordFormatDeclaration, RecordArtifactFile};

use crate::physical_runtime::recovery_construction::selected_rejoin::{
    resident::{
        discovery_allocation_denial, PhysicalRecoveryRejoinResidentDenial,
        StoreRejoinResidentLedger,
    },
    SelectedMediaRejoinDenial as Denial,
};

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) trait RouteWalkStorage {
    fn reserve_vec<T>(&mut self, count: usize) -> Result<Vec<T>, Denial>;
    fn grow_vec<T>(&mut self, values: &mut Vec<T>, additional: usize) -> Result<(), Denial>;
    fn grow_vec_geometrically<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), Denial>;
    fn discard_vec<T>(&mut self, values: Vec<T>) -> Result<(), Denial>;
    /// Format consumed and dropped a caller-funded Vec; release its measured
    /// capacity only after that constructor returns.
    fn release_consumed_bytes(&mut self, bytes: u64) -> Result<(), Denial>;
    /// Reads one page-sized artifact whole, under the page its format declares.
    fn read_page(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        format: PhysicalRecordFormatDeclaration,
        address: PageAddress,
    ) -> Result<ObservedRecoveryArtifact, Denial>;
    fn read_page_range(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        artifact: RecordArtifactFile,
        offset: u64,
        length: u32,
    ) -> Result<ObservedRecoveryArtifact, Denial>;
    fn discard_frame(&mut self, frame: ObservedRecoveryArtifact) -> Result<(), Denial>;
    fn overflow(&self) -> Denial;
}

impl RouteWalkStorage for () {
    fn reserve_vec<T>(&mut self, count: usize) -> Result<Vec<T>, Denial> {
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| Denial::BoundExceeded)?;
        Ok(values)
    }
    fn grow_vec<T>(&mut self, values: &mut Vec<T>, additional: usize) -> Result<(), Denial> {
        values
            .try_reserve_exact(additional)
            .map_err(|_| Denial::BoundExceeded)
    }
    fn grow_vec_geometrically<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), Denial> {
        self.grow_vec(values, additional)
    }
    fn discard_vec<T>(&mut self, values: Vec<T>) -> Result<(), Denial> {
        drop(values);
        Ok(())
    }
    fn release_consumed_bytes(&mut self, _bytes: u64) -> Result<(), Denial> {
        Ok(())
    }
    fn read_page(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        format: PhysicalRecordFormatDeclaration,
        address: PageAddress,
    ) -> Result<ObservedRecoveryArtifact, Denial> {
        let ceiling = ArtifactCeiling::page(format, address);
        discovery.read_with_allocator(ceiling, ReadGrant::ceiling_only(), |length| {
            let mut bytes = Vec::new();
            bytes.try_reserve_exact(length).map_err(|_| Denial::BoundExceeded)?;
            bytes.resize(length, 0);
            Ok(bytes)
        }).observed().map_err(|failure| match failure {
            worth_store_physical_backend::RecoveryDiscoveryAllocationFailure::Discovery(cause) => Denial::Discovery(cause),
            worth_store_physical_backend::RecoveryDiscoveryAllocationFailure::Allocation { cause, .. } => cause,
            worth_store_physical_backend::RecoveryDiscoveryAllocationFailure::BufferLengthMismatch {
                artifact, offset, requested, observed,
            } => Denial::ReadBufferLengthMismatch { artifact, offset, requested, observed },
        })
    }
    fn discard_frame(&mut self, frame: ObservedRecoveryArtifact) -> Result<(), Denial> {
        drop(frame);
        Ok(())
    }
    fn read_page_range(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        artifact: RecordArtifactFile,
        offset: u64,
        length: u32,
    ) -> Result<ObservedRecoveryArtifact, Denial> {
        discovery.read_record_range_with_allocator(artifact, offset, length, ReadGrant::ceiling_only(), |size| {
            let mut bytes = Vec::new();
            bytes.try_reserve_exact(size).map_err(|_| Denial::BoundExceeded)?;
            bytes.resize(size, 0);
            Ok(bytes)
        }).observed().map_err(|failure| match failure {
            worth_store_physical_backend::RecoveryDiscoveryAllocationFailure::Discovery(cause) => Denial::Discovery(cause),
            worth_store_physical_backend::RecoveryDiscoveryAllocationFailure::Allocation { cause, .. } => cause,
            worth_store_physical_backend::RecoveryDiscoveryAllocationFailure::BufferLengthMismatch {
                artifact, offset, requested, observed,
            } => Denial::ReadBufferLengthMismatch { artifact, offset, requested, observed },
        })
    }
    fn overflow(&self) -> Denial {
        Denial::BoundExceeded
    }
}

impl RouteWalkStorage for StoreRejoinResidentLedger {
    fn reserve_vec<T>(&mut self, count: usize) -> Result<Vec<T>, Denial> {
        StoreRejoinResidentLedger::reserve_vec(self, count).map_err(Denial::Resident)
    }
    fn grow_vec<T>(&mut self, values: &mut Vec<T>, additional: usize) -> Result<(), Denial> {
        StoreRejoinResidentLedger::grow_vec(self, values, additional).map_err(Denial::Resident)
    }
    fn grow_vec_geometrically<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), Denial> {
        StoreRejoinResidentLedger::grow_vec_geometrically(self, values, additional)
            .map_err(Denial::Resident)
    }
    fn discard_vec<T>(&mut self, values: Vec<T>) -> Result<(), Denial> {
        let bytes = self.vector_bytes(&values).map_err(Denial::Resident)?;
        drop(values);
        self.release(bytes);
        Ok(())
    }
    fn release_consumed_bytes(&mut self, bytes: u64) -> Result<(), Denial> {
        self.release(bytes);
        Ok(())
    }
    fn read_page(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        format: PhysicalRecordFormatDeclaration,
        address: PageAddress,
    ) -> Result<ObservedRecoveryArtifact, Denial> {
        self.transient(u64::from(format.page_size().bytes()))
            .map_err(Denial::Resident)?;
        discovery
            .read_with_allocator(
                ArtifactCeiling::page(format, address),
                ReadGrant::ceiling_only(),
                |length| self.reserve_bytes(length),
            )
            .observed()
            .map_err(discovery_allocation_denial)
    }
    fn discard_frame(&mut self, frame: ObservedRecoveryArtifact) -> Result<(), Denial> {
        let bytes = frame.owned_heap_bytes().ok_or_else(|| overflow(self))?;
        drop(frame);
        self.release(bytes);
        Ok(())
    }
    fn read_page_range(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        artifact: RecordArtifactFile,
        offset: u64,
        length: u32,
    ) -> Result<ObservedRecoveryArtifact, Denial> {
        self.transient(u64::from(length))
            .map_err(Denial::Resident)?;
        discovery
            .read_record_range_with_allocator(
                artifact,
                offset,
                length,
                ReadGrant::ceiling_only(),
                |size| self.reserve_bytes(size),
            )
            .observed()
            .map_err(discovery_allocation_denial)
    }
    fn overflow(&self) -> Denial {
        overflow(self)
    }
}

fn overflow(resident: &StoreRejoinResidentLedger) -> Denial {
    Denial::Resident(PhysicalRecoveryRejoinResidentDenial::SizeOverflow {
        admitted: resident.used().saturating_add(resident.remaining()),
    })
}
