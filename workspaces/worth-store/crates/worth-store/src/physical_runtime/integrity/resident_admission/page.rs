use sha2::{Digest, Sha256};
use worth_store_buffer_pool::{PhysicalFrameLease, PhysicalResidentFrameGeneration};
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};
use worth_store_physical_integrity::{
    validate_inline_page, InlinePageIntegrityValidation, InlineRecordProjectionDenial,
    IntegrityValidatedPageFrame, PhysicalArtifactScope,
};

use super::{
    denial::ResidentIntegrityAdmissionDenial, load::ResidentAdmissionContext,
    record_binding::ResidentIntegrityRecordBinding, source_scope::require_exact_page_source,
};

pub(in crate::physical_runtime) struct IntegrityAdmittedResidentPage<'frame> {
    source: ResidentIntegrityRecordBinding<'frame>,
}

pub(in crate::physical_runtime) struct IntegrityAdmittedResidentPageView<'frame> {
    lease: &'frame PhysicalFrameLease,
    scope: PhysicalArtifactScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct IntegrityAdmittedResidentPageBasis {
    page: worth_store_physical_format::PageGenerationCell,
    coordinate: RecordFrameCoordinate,
    page_lsn: worth_store_physical_format::PhysicalPageLsn,
    encoded_digest: [u8; 32],
}

pub(in crate::physical_runtime) fn admit_resident_page<'frame>(
    lease: &'frame PhysicalFrameLease,
    scope: PhysicalArtifactScope,
    expected_segment: RecordArtifactFile,
    context: ResidentAdmissionContext<'_>,
) -> Result<IntegrityAdmittedResidentPage<'frame>, ResidentIntegrityAdmissionDenial> {
    require_exact_page_source(lease, scope, expected_segment)
        .map_err(|denial| context.refuse_source(denial))?;
    if let Some(source) = context.reuse(lease, scope)? {
        return Ok(IntegrityAdmittedResidentPage { source });
    }
    let input = context.exact_input(lease, scope)?;
    context.observe_fresh_validation();
    match validate_inline_page(input, scope).0 {
        InlinePageIntegrityValidation::Intact(validated) => {
            bind_validated_page(lease, input, validated, context)
        }
        InlinePageIntegrityValidation::Rejected(rejection) => {
            context.validation_rejected(rejection)
        }
    }
}

fn bind_validated_page<'frame>(
    lease: &'frame PhysicalFrameLease,
    input: worth_store_physical_integrity::UntrustedPhysicalArtifact<'frame>,
    validated: IntegrityValidatedPageFrame<'frame>,
    context: ResidentAdmissionContext<'_>,
) -> Result<IntegrityAdmittedResidentPage<'frame>, ResidentIntegrityAdmissionDenial> {
    if !validated.matches_input(input) {
        return context.deny(ResidentIntegrityAdmissionDenial::SourceIncarnationMismatch);
    }
    let scope = validated.scope();
    let source = context.bind_validated(lease, scope, validated.into_validation_record())?;
    Ok(IntegrityAdmittedResidentPage { source })
}

impl<'frame> IntegrityAdmittedResidentPage<'frame> {
    pub(in crate::physical_runtime) fn with_owner_decoder<T>(
        self,
        context: ResidentAdmissionContext<'_>,
        decoder: impl for<'view> FnOnce(IntegrityAdmittedResidentPageView<'view>) -> T,
    ) -> Result<T, ResidentIntegrityAdmissionDenial> {
        context.with_owner_decoder(self.source, |lease, scope| {
            decoder(IntegrityAdmittedResidentPageView { lease, scope })
        })
    }
}

impl IntegrityAdmittedResidentPageView<'_> {
    pub(in crate::physical_runtime) fn project_page(
        &self,
    ) -> Result<ResidentInlinePageProjection, InlineRecordProjectionDenial> {
        let bytes = &self.lease[..];
        let header = worth_store_physical_format::DURABLE_FRAME_HEADER_BYTES;
        let prefix = worth_store_physical_format::DURABLE_INLINE_PAGE_PREFIX_BYTES;
        let slot_bytes = worth_store_physical_format::DURABLE_INLINE_SLOT_BYTES;
        let count = read_u16(bytes, header + 16)
            .ok_or(InlineRecordProjectionDenial::SlotIdentityMismatch)?;
        let directory_end = prefix + usize::from(count) * slot_bytes;
        let payload_len = bytes
            .len()
            .checked_sub(header)
            .ok_or(InlineRecordProjectionDenial::PayloadLengthMismatch)?;
        let data_start = if count == 0 {
            payload_len
        } else {
            let last = header + prefix + (usize::from(count) - 1) * slot_bytes;
            read_u32(bytes, last + 24).ok_or(InlineRecordProjectionDenial::PayloadLengthMismatch)?
                as usize
        };
        let free_bytes = data_start
            .checked_sub(directory_end)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(InlineRecordProjectionDenial::PayloadLengthMismatch)?;
        let records = (0..count)
            .map(|index| project_record_descriptor(bytes, index))
            .collect::<Result<Vec<_>, _>>()?;
        let page = self
            .scope
            .page_identity()
            .ok_or(InlineRecordProjectionDenial::PageIdentityMismatch)?;
        let page_lsn =
            admitted_page_lsn(bytes).ok_or(InlineRecordProjectionDenial::PayloadLengthMismatch)?;
        let encoded_digest = Sha256::digest(bytes).into();
        Ok(ResidentInlinePageProjection {
            page,
            slot_count: count,
            free_bytes,
            records,
            prior_basis: IntegrityAdmittedResidentPageBasis {
                page,
                coordinate: self.lease.key().coordinate(),
                page_lsn,
                encoded_digest,
            },
        })
    }

    pub(in crate::physical_runtime) fn project_record(
        &self,
        placement: worth_store_physical_format::DurableInlineRecordPlacement,
    ) -> Result<ResidentInlineRecordProjection, InlineRecordProjectionDenial> {
        if self.scope.page_identity() != Some(placement.page_cell()) {
            return Err(InlineRecordProjectionDenial::PageIdentityMismatch);
        }
        let slot_index = usize::from(placement.slot().get() - 1);
        let slot_base = worth_store_physical_format::DURABLE_FRAME_HEADER_BYTES
            + worth_store_physical_format::DURABLE_INLINE_PAGE_PREFIX_BYTES
            + slot_index * worth_store_physical_format::DURABLE_INLINE_SLOT_BYTES;
        let bytes = &self.lease[..];
        let slot = bytes
            .get(slot_base..slot_base + worth_store_physical_format::DURABLE_INLINE_SLOT_BYTES)
            .ok_or(InlineRecordProjectionDenial::SlotIdentityMismatch)?;
        let record = worth_store_physical_format::PersistedRecordIdentity::new(
            slot[..16]
                .try_into()
                .map_err(|_| InlineRecordProjectionDenial::RecordIdentityMismatch)?,
            read_u64(slot, 16).ok_or(InlineRecordProjectionDenial::RecordIdentityMismatch)?,
        )
        .ok_or(InlineRecordProjectionDenial::RecordIdentityMismatch)?;
        if record != placement.record() {
            return Err(InlineRecordProjectionDenial::RecordIdentityMismatch);
        }
        if read_u64(slot, 32) != Some(placement.slot_generation()) {
            return Err(InlineRecordProjectionDenial::SlotGenerationMismatch);
        }
        let offset =
            read_u32(slot, 24).ok_or(InlineRecordProjectionDenial::PayloadLengthMismatch)? as usize;
        let length =
            read_u32(slot, 28).ok_or(InlineRecordProjectionDenial::PayloadLengthMismatch)? as usize;
        if length as u64 != placement.payload_bytes() {
            return Err(InlineRecordProjectionDenial::PayloadLengthMismatch);
        }
        let start = worth_store_physical_format::DURABLE_FRAME_HEADER_BYTES + offset;
        let end = start
            .checked_add(length)
            .filter(|end| *end <= bytes.len())
            .ok_or(InlineRecordProjectionDenial::PayloadLengthMismatch)?;
        Ok(ResidentInlineRecordProjection {
            payload: start..end,
            page_lsn: admitted_page_lsn(bytes)
                .ok_or(InlineRecordProjectionDenial::PayloadLengthMismatch)?,
        })
    }

    pub(in crate::physical_runtime) const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }

    pub(in crate::physical_runtime) const fn frame_generation(
        &self,
    ) -> PhysicalResidentFrameGeneration {
        self.lease.resident_generation()
    }
}

pub(in crate::physical_runtime) struct ResidentInlineRecordProjection {
    pub(in crate::physical_runtime) payload: std::ops::Range<usize>,
    pub(in crate::physical_runtime) page_lsn: worth_store_physical_format::PhysicalPageLsn,
}

pub(in crate::physical_runtime) struct ResidentInlinePageProjection {
    pub(in crate::physical_runtime) page: worth_store_physical_format::PageGenerationCell,
    pub(in crate::physical_runtime) slot_count: u16,
    pub(in crate::physical_runtime) free_bytes: u32,
    pub(in crate::physical_runtime) records: Vec<ResidentInlinePageRecordProjection>,
    pub(in crate::physical_runtime) prior_basis: IntegrityAdmittedResidentPageBasis,
}

impl IntegrityAdmittedResidentPageBasis {
    pub(in crate::physical_runtime) const fn page(
        self,
    ) -> worth_store_physical_format::PageGenerationCell {
        self.page
    }

    pub(in crate::physical_runtime) const fn coordinate(self) -> RecordFrameCoordinate {
        self.coordinate
    }

    pub(in crate::physical_runtime) const fn page_lsn(
        self,
    ) -> worth_store_physical_format::PhysicalPageLsn {
        self.page_lsn
    }

    pub(in crate::physical_runtime) const fn encoded_digest(self) -> [u8; 32] {
        self.encoded_digest
    }
}

pub(in crate::physical_runtime) struct ResidentInlinePageRecordProjection {
    pub(in crate::physical_runtime) record: worth_store_physical_format::PersistedRecordIdentity,
    pub(in crate::physical_runtime) slot: worth_store_physical_format::PhysicalRecordSlot,
    pub(in crate::physical_runtime) slot_generation: u64,
    pub(in crate::physical_runtime) payload_bytes: u32,
}

fn project_record_descriptor(
    bytes: &[u8],
    index: u16,
) -> Result<ResidentInlinePageRecordProjection, InlineRecordProjectionDenial> {
    let base = worth_store_physical_format::DURABLE_FRAME_HEADER_BYTES
        + worth_store_physical_format::DURABLE_INLINE_PAGE_PREFIX_BYTES
        + usize::from(index) * worth_store_physical_format::DURABLE_INLINE_SLOT_BYTES;
    let record = worth_store_physical_format::PersistedRecordIdentity::new(
        bytes
            .get(base..base + 16)
            .and_then(|value| value.try_into().ok())
            .ok_or(InlineRecordProjectionDenial::RecordIdentityMismatch)?,
        read_u64(bytes, base + 16).ok_or(InlineRecordProjectionDenial::RecordIdentityMismatch)?,
    )
    .ok_or(InlineRecordProjectionDenial::RecordIdentityMismatch)?;
    Ok(ResidentInlinePageRecordProjection {
        record,
        slot: worth_store_physical_format::PhysicalRecordSlot::from_raw(index + 1)
            .map_err(|_| InlineRecordProjectionDenial::SlotIdentityMismatch)?,
        slot_generation: read_u64(bytes, base + 32)
            .ok_or(InlineRecordProjectionDenial::SlotGenerationMismatch)?,
        payload_bytes: read_u32(bytes, base + 28)
            .ok_or(InlineRecordProjectionDenial::PayloadLengthMismatch)?,
    })
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        bytes.get(offset..offset + 8)?.try_into().ok()?,
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn admitted_page_lsn(bytes: &[u8]) -> Option<worth_store_physical_format::PhysicalPageLsn> {
    Some(worth_store_physical_format::PhysicalPageLsn::new(
        u64::from_le_bytes(bytes.get(36..44)?.try_into().ok()?),
    ))
}
