use worth_store_physical_format::{
    DurableExtentManifest, ExtentChunkCoordinate, PersistedRecordIdentity,
    PhysicalRecordFormatDeclaration, RecordExtentGenerationCell,
};

use super::super::{
    PhysicalArtifactScope, PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    PhysicalIntegrityValidationRecord, UntrustedPhysicalArtifact,
};

#[derive(Debug)]
pub struct IntegrityValidatedExtentManifest<'media> {
    scope: PhysicalArtifactScope,
    record_format: PhysicalRecordFormatDeclaration,
    record: PersistedRecordIdentity,
    extent: RecordExtentGenerationCell,
    logical_bytes: u64,
    maximum_frame_bytes: u32,
    chunk_payload_capacity: u32,
    chunk_count: u32,
    validation_record: PhysicalIntegrityValidationRecord,
    inspected: UntrustedPhysicalArtifact<'media>,
}

/// Byte-free membership facts projected from one validated extent manifest.
/// It can constrain chunk validation but grants no access to chunk bytes.
#[derive(Debug, Clone, Copy)]
pub struct IntegrityValidatedExtentMembership {
    scope: PhysicalArtifactScope,
    record_format: PhysicalRecordFormatDeclaration,
    record: PersistedRecordIdentity,
    extent: RecordExtentGenerationCell,
    logical_bytes: u64,
    chunk_payload_capacity: u32,
    chunk_count: u32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ExtentManifestChunkMembership {
    coordinate: ExtentChunkCoordinate,
    payload_bytes: u64,
}

impl<'media> IntegrityValidatedExtentManifest<'media> {
    pub(crate) fn new(
        scope: PhysicalArtifactScope,
        manifest: DurableExtentManifest,
        record_format: PhysicalRecordFormatDeclaration,
        validated_range_checksum: u32,
        inspected: UntrustedPhysicalArtifact<'media>,
    ) -> Option<Self> {
        let placement = scope.extent_manifest_placement()?;
        if !scope.is_extent_manifest()
            || record_format != scope.record_format()
            || manifest.record() != placement.record()
            || manifest.extent_cell() != placement.extent_cell()
            || manifest.logical_bytes() != placement.payload_bytes()
            || inspected.byte_count() != scope.byte_range().length()
        {
            return None;
        }
        let validation_record = PhysicalIntegrityValidationRecord::from_validated_scope(
            scope,
            PhysicalIntegrityValidationDigest::crc32c(scope.exact_extent_scope_digest()),
            PhysicalIntegrityValidationDigest::crc32c(validated_range_checksum),
            PhysicalIntegrityValidationMechanism::Crc32cV1,
        )?;
        Some(Self {
            scope,
            record_format,
            record: manifest.record(),
            extent: manifest.extent_cell(),
            logical_bytes: manifest.logical_bytes(),
            maximum_frame_bytes: manifest.maximum_frame_bytes(),
            chunk_payload_capacity: manifest.chunk_payload_capacity(),
            chunk_count: manifest.chunk_count(),
            validation_record,
            inspected,
        })
    }

    pub const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn record_format(&self) -> PhysicalRecordFormatDeclaration {
        self.record_format
    }

    pub const fn record(&self) -> PersistedRecordIdentity {
        self.record
    }

    pub const fn extent_cell(&self) -> RecordExtentGenerationCell {
        self.extent
    }

    pub const fn logical_bytes(&self) -> u64 {
        self.logical_bytes
    }

    pub const fn maximum_frame_bytes(&self) -> u32 {
        self.maximum_frame_bytes
    }

    pub const fn chunk_payload_capacity(&self) -> u32 {
        self.chunk_payload_capacity
    }

    pub const fn chunk_count(&self) -> u32 {
        self.chunk_count
    }

    pub const fn membership(&self) -> IntegrityValidatedExtentMembership {
        IntegrityValidatedExtentMembership {
            scope: self.scope,
            record_format: self.record_format,
            record: self.record,
            extent: self.extent,
            logical_bytes: self.logical_bytes,
            chunk_payload_capacity: self.chunk_payload_capacity,
            chunk_count: self.chunk_count,
        }
    }

    pub const fn into_validation_record(self) -> PhysicalIntegrityValidationRecord {
        self.validation_record
    }

    pub fn matches_input(&self, input: UntrustedPhysicalArtifact<'media>) -> bool {
        self.inspected.same_incarnation(input)
    }
}

impl IntegrityValidatedExtentMembership {
    pub fn from_validation_record(
        record: PhysicalIntegrityValidationRecord,
        scope: PhysicalArtifactScope,
    ) -> Option<Self> {
        let placement = scope.extent_manifest_placement()?;
        if !record.matches_scope(scope) {
            return None;
        }
        let maximum_frame_bytes = scope.record_format().page_size().bytes();
        let overhead = worth_store_physical_format::DURABLE_EXTENT_FRAME_HEADER_BYTES
            + worth_store_physical_format::EXTENT_CHUNK_METADATA_BYTES;
        let chunk_payload_capacity = maximum_frame_bytes.checked_sub(overhead as u32)?;
        let chunk_count = u32::try_from(
            placement
                .payload_bytes()
                .div_ceil(u64::from(chunk_payload_capacity)),
        )
        .ok()?;
        Some(Self {
            scope,
            record_format: scope.record_format(),
            record: placement.record(),
            extent: placement.extent_cell(),
            logical_bytes: placement.payload_bytes(),
            chunk_payload_capacity,
            chunk_count,
        })
    }

    pub(crate) const fn scope(self) -> PhysicalArtifactScope {
        self.scope
    }

    pub(crate) const fn record_format(self) -> PhysicalRecordFormatDeclaration {
        self.record_format
    }

    pub(crate) const fn record(self) -> PersistedRecordIdentity {
        self.record
    }

    pub(crate) const fn extent_cell(self) -> RecordExtentGenerationCell {
        self.extent
    }

    pub(crate) const fn logical_bytes(self) -> u64 {
        self.logical_bytes
    }

    pub(crate) fn chunk_membership(self, ordinal: u32) -> Option<ExtentManifestChunkMembership> {
        if ordinal == 0 || ordinal > self.chunk_count {
            return None;
        }
        let logical_offset =
            u64::from(ordinal - 1).checked_mul(u64::from(self.chunk_payload_capacity))?;
        let payload_bytes = self
            .logical_bytes
            .checked_sub(logical_offset)?
            .min(u64::from(self.chunk_payload_capacity));
        Some(ExtentManifestChunkMembership {
            coordinate: ExtentChunkCoordinate::new(
                self.record,
                self.extent,
                self.logical_bytes,
                logical_offset,
                ordinal,
            )?,
            payload_bytes,
        })
    }
}

impl ExtentManifestChunkMembership {
    pub(crate) const fn coordinate(self) -> ExtentChunkCoordinate {
        self.coordinate
    }

    pub(crate) const fn payload_bytes(self) -> u64 {
        self.payload_bytes
    }
}
