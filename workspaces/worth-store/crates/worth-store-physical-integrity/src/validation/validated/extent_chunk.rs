use std::ops::Range;

use worth_store_physical_format::{
    ExtentChunkCoordinate, PersistedRecordIdentity, PhysicalPageLsn,
    PhysicalRecordFormatDeclaration, RecordExtentGenerationCell, DURABLE_EXTENT_FRAME_HEADER_BYTES,
    EXTENT_CHUNK_METADATA_BYTES,
};

use super::super::{
    PhysicalArtifactScope, PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    PhysicalIntegrityValidationRecord, UntrustedPhysicalArtifact,
};

#[derive(Debug)]
pub struct IntegrityValidatedExtentChunkFrame<'media> {
    scope: PhysicalArtifactScope,
    record_format: PhysicalRecordFormatDeclaration,
    coordinate: ExtentChunkCoordinate,
    payload_range: Range<usize>,
    page_lsn: PhysicalPageLsn,
    validation_record: PhysicalIntegrityValidationRecord,
    inspected: UntrustedPhysicalArtifact<'media>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtentChunkProjectionDenial {
    InputIncarnationMismatch,
    RecordIdentityMismatch,
    ExtentIdentityMismatch,
    ExtentGenerationMismatch,
    LogicalLengthMismatch,
    LogicalOffsetMismatch,
    ChunkOrdinalMismatch,
}

#[derive(Debug)]
pub struct IntegrityValidatedExtentChunkProjection<'view, 'media> {
    validated: &'view IntegrityValidatedExtentChunkFrame<'media>,
}

impl<'media> IntegrityValidatedExtentChunkFrame<'media> {
    pub(crate) fn new(
        scope: PhysicalArtifactScope,
        record_format: PhysicalRecordFormatDeclaration,
        chunk_bytes: &'media [u8],
        validated_range_checksum: u32,
        inspected: UntrustedPhysicalArtifact<'media>,
    ) -> Option<Self> {
        let coordinate = scope.extent_chunk_coordinate()?;
        if !scope.is_extent_chunk()
            || record_format != scope.record_format()
            || inspected.byte_count() != scope.byte_range().length()
        {
            return None;
        }
        let inspected_tail = inspected
            .bytes()
            .get(inspected.bytes().len().checked_sub(chunk_bytes.len())?..)?;
        if inspected_tail.len() != chunk_bytes.len()
            || !core::ptr::eq(inspected_tail.as_ptr(), chunk_bytes.as_ptr())
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
            coordinate,
            payload_range: DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES
                ..DURABLE_EXTENT_FRAME_HEADER_BYTES
                    + EXTENT_CHUNK_METADATA_BYTES
                    + chunk_bytes.len(),
            page_lsn: super::data_frame_projection::page_lsn(inspected)?,
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

    pub const fn coordinate(&self) -> ExtentChunkCoordinate {
        self.coordinate
    }

    pub const fn record(&self) -> PersistedRecordIdentity {
        self.coordinate.record()
    }

    pub const fn extent_cell(&self) -> RecordExtentGenerationCell {
        self.coordinate.extent_cell()
    }

    pub const fn logical_bytes(&self) -> u64 {
        self.coordinate.logical_bytes()
    }

    pub const fn logical_offset(&self) -> u64 {
        self.coordinate.logical_offset()
    }

    pub const fn ordinal(&self) -> u32 {
        self.coordinate.ordinal()
    }

    pub const fn page_lsn(&self) -> PhysicalPageLsn {
        self.page_lsn
    }

    pub fn project_chunk<'view>(
        &'view self,
        input: UntrustedPhysicalArtifact<'media>,
        expected: ExtentChunkCoordinate,
    ) -> Result<IntegrityValidatedExtentChunkProjection<'view, 'media>, ExtentChunkProjectionDenial>
    {
        if !self.inspected.same_incarnation(input) {
            return Err(ExtentChunkProjectionDenial::InputIncarnationMismatch);
        }
        if expected.record() != self.coordinate.record() {
            return Err(ExtentChunkProjectionDenial::RecordIdentityMismatch);
        }
        if expected.extent_cell().extent_id() != self.coordinate.extent_cell().extent_id() {
            return Err(ExtentChunkProjectionDenial::ExtentIdentityMismatch);
        }
        if expected.extent_cell().generation() != self.coordinate.extent_cell().generation() {
            return Err(ExtentChunkProjectionDenial::ExtentGenerationMismatch);
        }
        if expected.logical_bytes() != self.coordinate.logical_bytes() {
            return Err(ExtentChunkProjectionDenial::LogicalLengthMismatch);
        }
        if expected.logical_offset() != self.coordinate.logical_offset() {
            return Err(ExtentChunkProjectionDenial::LogicalOffsetMismatch);
        }
        if expected.ordinal() != self.coordinate.ordinal() {
            return Err(ExtentChunkProjectionDenial::ChunkOrdinalMismatch);
        }
        Ok(IntegrityValidatedExtentChunkProjection { validated: self })
    }

    pub const fn into_validation_record(self) -> PhysicalIntegrityValidationRecord {
        self.validation_record
    }

    pub fn matches_input(&self, input: UntrustedPhysicalArtifact<'media>) -> bool {
        self.inspected.same_incarnation(input)
    }
}

impl IntegrityValidatedExtentChunkProjection<'_, '_> {
    pub const fn coordinate(&self) -> ExtentChunkCoordinate {
        self.validated.coordinate()
    }

    pub fn payload_range(&self) -> Range<usize> {
        self.validated.payload_range.clone()
    }

    pub const fn page_lsn(&self) -> PhysicalPageLsn {
        self.validated.page_lsn()
    }
}
