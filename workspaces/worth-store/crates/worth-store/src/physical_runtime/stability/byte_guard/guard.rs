use super::{ByteGuardReleaseReceipt, PhysicalByteGuardDenial, PhysicalByteGuardScope};
use crate::physical_runtime::stability::PhysicalByteGuardAdmission;
use crate::physical_runtime::PhysicalRecordChunkView;
use worth_store_physical_isolation::PhysicalReadProtectedFootprintBasis;

pub struct PhysicalByteGuard<'a> {
    scope: PhysicalByteGuardScope,
    footprint_basis: PhysicalReadProtectedFootprintBasis,
    bytes: GuardedPhysicalBytes<'a>,
}

enum GuardedPhysicalBytes<'a> {
    RecordChunk(PhysicalRecordChunkView<'a>),
}

impl std::fmt::Debug for PhysicalByteGuard<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PhysicalByteGuard")
            .field("scope", &self.scope)
            .field("footprint_basis", &self.footprint_basis)
            .field("guarded_bytes", &self.bytes_for_execution().len())
            .finish()
    }
}

impl<'a> PhysicalByteGuard<'a> {
    pub fn from_record_chunk(
        admission: PhysicalByteGuardAdmission,
        chunk: PhysicalRecordChunkView<'a>,
    ) -> Result<Self, PhysicalByteGuardDenial> {
        let scope = admission.scope();
        if scope.chunk_basis() != chunk.basis() {
            return Err(PhysicalByteGuardDenial::StoreChunkBasisMismatch {
                expected: scope.chunk_basis(),
                observed: chunk.basis(),
            });
        }
        Ok(Self {
            scope,
            footprint_basis: admission.footprint_basis(),
            bytes: GuardedPhysicalBytes::RecordChunk(chunk),
        })
    }

    pub const fn scope(&self) -> PhysicalByteGuardScope {
        self.scope
    }

    pub const fn footprint_basis(&self) -> PhysicalReadProtectedFootprintBasis {
        self.footprint_basis
    }

    pub(crate) fn bytes_for_execution(&self) -> &[u8] {
        match &self.bytes {
            GuardedPhysicalBytes::RecordChunk(chunk) => chunk.bytes(),
        }
    }

    pub fn release(self) -> ByteGuardReleaseReceipt {
        ByteGuardReleaseReceipt::new(self.scope, self.bytes_for_execution().len() as u64)
    }
}
