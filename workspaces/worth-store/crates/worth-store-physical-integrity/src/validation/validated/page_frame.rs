use std::ops::Range;

use worth_store_physical_format::{
    DurableInlineRecordPlacement, InlinePageGeometry, PageGenerationCell, PersistedRecordIdentity,
    PhysicalPageLsn, PhysicalRecordFormatDeclaration, SlotGenerationCell,
    DURABLE_FRAME_HEADER_BYTES, DURABLE_INLINE_PAGE_PREFIX_BYTES, DURABLE_INLINE_SLOT_BYTES,
};

use super::super::{
    PhysicalArtifactScope, PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    PhysicalIntegrityValidationRecord, UntrustedPhysicalArtifact,
};

#[derive(Debug)]
pub struct IntegrityValidatedPageFrame<'media> {
    scope: PhysicalArtifactScope,
    record_format: PhysicalRecordFormatDeclaration,
    page: PageGenerationCell,
    slot_count: u16,
    free_bytes: u32,
    page_lsn: PhysicalPageLsn,
    validation_record: PhysicalIntegrityValidationRecord,
    inspected: UntrustedPhysicalArtifact<'media>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineRecordProjectionDenial {
    InputIncarnationMismatch,
    PageIdentityMismatch,
    SlotIdentityMismatch,
    RecordIdentityMismatch,
    SlotGenerationMismatch,
    PayloadLengthMismatch,
}

#[derive(Debug)]
pub struct IntegrityValidatedInlineRecordProjection<'view, 'media> {
    validated: &'view IntegrityValidatedPageFrame<'media>,
    record: PersistedRecordIdentity,
    slot: SlotGenerationCell,
    payload_range: Range<usize>,
}

impl<'media> IntegrityValidatedPageFrame<'media> {
    pub(crate) fn new(
        scope: PhysicalArtifactScope,
        geometry: InlinePageGeometry,
        validated_range_checksum: u32,
        inspected: UntrustedPhysicalArtifact<'media>,
    ) -> Option<Self> {
        if scope.page_identity()? != geometry.page_cell()
            || inspected.byte_count() != scope.byte_range().length()
            || inspected.byte_count() != u64::from(scope.record_format().page_size().bytes())
        {
            return None;
        }
        let validation_record = PhysicalIntegrityValidationRecord::from_validated_scope(
            scope,
            PhysicalIntegrityValidationDigest::crc32c(scope.exact_page_scope_digest()),
            PhysicalIntegrityValidationDigest::crc32c(validated_range_checksum),
            PhysicalIntegrityValidationMechanism::Crc32cV1,
        )?;
        Some(Self {
            scope,
            record_format: scope.record_format(),
            page: geometry.page_cell(),
            slot_count: geometry.slot_count(),
            free_bytes: geometry.free_bytes(),
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

    pub const fn page_identity(&self) -> PageGenerationCell {
        self.page
    }

    pub const fn slot_count(&self) -> u16 {
        self.slot_count
    }

    pub const fn free_bytes(&self) -> u32 {
        self.free_bytes
    }

    pub const fn page_lsn(&self) -> PhysicalPageLsn {
        self.page_lsn
    }

    pub fn project_record<'view>(
        &'view self,
        input: UntrustedPhysicalArtifact<'media>,
        placement: DurableInlineRecordPlacement,
    ) -> Result<IntegrityValidatedInlineRecordProjection<'view, 'media>, InlineRecordProjectionDenial>
    {
        if !self.inspected.same_incarnation(input) {
            return Err(InlineRecordProjectionDenial::InputIncarnationMismatch);
        }
        if placement.page_cell() != self.page {
            return Err(InlineRecordProjectionDenial::PageIdentityMismatch);
        }
        let slot_index = usize::from(placement.slot().get() - 1);
        if slot_index >= usize::from(self.slot_count) {
            return Err(InlineRecordProjectionDenial::SlotIdentityMismatch);
        }
        let slot_base = DURABLE_FRAME_HEADER_BYTES
            + DURABLE_INLINE_PAGE_PREFIX_BYTES
            + slot_index * DURABLE_INLINE_SLOT_BYTES;
        let bytes = self.inspected.bytes();
        let record = PersistedRecordIdentity::new(
            bytes[slot_base..slot_base + 16]
                .try_into()
                .expect("validated inline slot has a fixed record epoch"),
            read_u64(bytes, slot_base + 16),
        )
        .expect("validated inline slot has a canonical record identity");
        if record != placement.record() {
            return Err(InlineRecordProjectionDenial::RecordIdentityMismatch);
        }
        if read_u64(bytes, slot_base + 32) != placement.slot_generation() {
            return Err(InlineRecordProjectionDenial::SlotGenerationMismatch);
        }
        let payload_offset = read_u32(bytes, slot_base + 24) as usize;
        let payload_bytes = read_u32(bytes, slot_base + 28) as usize;
        if payload_bytes as u64 != placement.payload_bytes() {
            return Err(InlineRecordProjectionDenial::PayloadLengthMismatch);
        }
        let payload_start = DURABLE_FRAME_HEADER_BYTES + payload_offset;
        let payload_end = payload_start + payload_bytes;
        debug_assert!(payload_end <= bytes.len());
        Ok(IntegrityValidatedInlineRecordProjection {
            validated: self,
            record,
            slot: placement.slot_cell(),
            payload_range: payload_start..payload_end,
        })
    }

    pub const fn into_validation_record(self) -> PhysicalIntegrityValidationRecord {
        self.validation_record
    }

    /// Matches the exact immutable slice incarnation inspected by validation.
    /// It exposes no bytes and grants no decoder authority.
    pub fn matches_input(&self, input: UntrustedPhysicalArtifact<'media>) -> bool {
        self.inspected.same_incarnation(input)
    }
}

impl IntegrityValidatedInlineRecordProjection<'_, '_> {
    pub const fn record(&self) -> PersistedRecordIdentity {
        self.record
    }

    pub const fn page_identity(&self) -> PageGenerationCell {
        self.validated.page_identity()
    }

    pub const fn slot_identity(&self) -> SlotGenerationCell {
        self.slot
    }

    pub fn payload_range(&self) -> Range<usize> {
        self.payload_range.clone()
    }

    pub const fn page_lsn(&self) -> PhysicalPageLsn {
        self.validated.page_lsn()
    }
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        bytes[offset..offset + 8]
            .try_into()
            .expect("validated inline slot has a fixed u64 field"),
    )
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("validated inline slot has a fixed u32 field"),
    )
}
