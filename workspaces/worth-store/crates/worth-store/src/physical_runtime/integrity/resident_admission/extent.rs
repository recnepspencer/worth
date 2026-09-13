use worth_store_buffer_pool::PhysicalFrameLease;
use worth_store_physical_integrity::{
    validate_extent_chunk_membership, validate_extent_manifest, ExtentChunkIntegrityValidation,
    ExtentChunkProjectionDenial, ExtentManifestIntegrityValidation,
    IntegrityValidatedExtentChunkFrame, IntegrityValidatedExtentManifest,
    IntegrityValidatedExtentMembership, PhysicalArtifactScope, UntrustedPhysicalArtifact,
};

use super::{
    denial::ResidentIntegrityAdmissionDenial, load::ResidentAdmissionContext,
    record_binding::ResidentIntegrityRecordBinding,
};

pub(in crate::physical_runtime) struct IntegrityAdmittedResidentExtentManifest<'frame> {
    source: ResidentIntegrityRecordBinding<'frame>,
    membership: IntegrityValidatedExtentMembership,
}

pub(in crate::physical_runtime) struct IntegrityAdmittedResidentExtentChunk<'frame> {
    source: ResidentIntegrityRecordBinding<'frame>,
}

pub(in crate::physical_runtime) struct IntegrityAdmittedResidentExtentManifestView<'frame> {
    lease: &'frame PhysicalFrameLease,
    scope: PhysicalArtifactScope,
}

pub(in crate::physical_runtime) struct IntegrityAdmittedResidentExtentChunkView<'frame> {
    lease: &'frame PhysicalFrameLease,
    scope: PhysicalArtifactScope,
}

pub(in crate::physical_runtime) fn admit_resident_extent_manifest<'frame>(
    lease: &'frame PhysicalFrameLease,
    scope: PhysicalArtifactScope,
    context: ResidentAdmissionContext<'_>,
) -> Result<IntegrityAdmittedResidentExtentManifest<'frame>, ResidentIntegrityAdmissionDenial> {
    if let Some(source) = context.reuse(lease, scope)? {
        let Some(membership) = IntegrityValidatedExtentMembership::from_validation_record(
            source.validation_record(),
            scope,
        ) else {
            return context.deny(ResidentIntegrityAdmissionDenial::RetainedRecordChanged);
        };
        return Ok(IntegrityAdmittedResidentExtentManifest { source, membership });
    }
    let input = context.exact_input(lease, scope)?;
    context.observe_fresh_validation();
    match validate_extent_manifest(input, scope).0 {
        ExtentManifestIntegrityValidation::Intact(validated) => {
            bind_extent_manifest(lease, input, validated, context)
        }
        ExtentManifestIntegrityValidation::Rejected(rejection) => {
            context.validation_rejected(rejection)
        }
    }
}

pub(in crate::physical_runtime) fn admit_resident_extent_chunk<'frame>(
    lease: &'frame PhysicalFrameLease,
    scope: PhysicalArtifactScope,
    manifest: IntegrityValidatedExtentMembership,
    context: ResidentAdmissionContext<'_>,
) -> Result<IntegrityAdmittedResidentExtentChunk<'frame>, ResidentIntegrityAdmissionDenial> {
    if let Some(source) = context.reuse(lease, scope)? {
        return Ok(IntegrityAdmittedResidentExtentChunk { source });
    }
    let input = context.exact_input(lease, scope)?;
    context.observe_fresh_validation();
    match validate_extent_chunk_membership(input, scope, manifest).0 {
        ExtentChunkIntegrityValidation::Intact(validated) => {
            bind_extent_chunk(lease, input, validated, context)
        }
        ExtentChunkIntegrityValidation::Rejected(rejection) => {
            context.validation_rejected(rejection)
        }
    }
}

fn bind_extent_manifest<'frame>(
    lease: &'frame PhysicalFrameLease,
    input: UntrustedPhysicalArtifact<'frame>,
    validated: IntegrityValidatedExtentManifest<'frame>,
    context: ResidentAdmissionContext<'_>,
) -> Result<IntegrityAdmittedResidentExtentManifest<'frame>, ResidentIntegrityAdmissionDenial> {
    if !validated.matches_input(input) {
        return context.deny(ResidentIntegrityAdmissionDenial::SourceIncarnationMismatch);
    }
    let scope = validated.scope();
    let membership = validated.membership();
    let source = context.bind_validated(lease, scope, validated.into_validation_record())?;
    Ok(IntegrityAdmittedResidentExtentManifest { source, membership })
}

fn bind_extent_chunk<'frame>(
    lease: &'frame PhysicalFrameLease,
    input: UntrustedPhysicalArtifact<'frame>,
    validated: IntegrityValidatedExtentChunkFrame<'frame>,
    context: ResidentAdmissionContext<'_>,
) -> Result<IntegrityAdmittedResidentExtentChunk<'frame>, ResidentIntegrityAdmissionDenial> {
    if !validated.matches_input(input) {
        return context.deny(ResidentIntegrityAdmissionDenial::SourceIncarnationMismatch);
    }
    let scope = validated.scope();
    let source = context.bind_validated(lease, scope, validated.into_validation_record())?;
    Ok(IntegrityAdmittedResidentExtentChunk { source })
}

impl<'frame> IntegrityAdmittedResidentExtentManifest<'frame> {
    pub(in crate::physical_runtime) const fn membership(
        &self,
    ) -> IntegrityValidatedExtentMembership {
        self.membership
    }

    pub(in crate::physical_runtime) fn with_owner_decoder<T>(
        self,
        context: ResidentAdmissionContext<'_>,
        decoder: impl for<'view> FnOnce(IntegrityAdmittedResidentExtentManifestView<'view>) -> T,
    ) -> Result<T, ResidentIntegrityAdmissionDenial> {
        context.with_owner_decoder(self.source, |lease, scope| {
            decoder(IntegrityAdmittedResidentExtentManifestView { lease, scope })
        })
    }
}

impl<'frame> IntegrityAdmittedResidentExtentChunk<'frame> {
    pub(in crate::physical_runtime) fn with_owner_decoder<T>(
        self,
        context: ResidentAdmissionContext<'_>,
        decoder: impl for<'view> FnOnce(IntegrityAdmittedResidentExtentChunkView<'view>) -> T,
    ) -> Result<T, ResidentIntegrityAdmissionDenial> {
        context.with_owner_decoder(self.source, |lease, scope| {
            decoder(IntegrityAdmittedResidentExtentChunkView { lease, scope })
        })
    }
}

impl IntegrityAdmittedResidentExtentManifestView<'_> {
    pub(in crate::physical_runtime) fn project_manifest(
        &self,
    ) -> Option<worth_store_physical_format::DurableExtentManifest> {
        let placement = self.scope.extent_manifest_placement()?;
        let format = self.scope.durable_frame_record_format()?;
        let maximum_frame_bytes = format.page_size().bytes();
        let overhead = worth_store_physical_format::DURABLE_EXTENT_FRAME_HEADER_BYTES
            + worth_store_physical_format::EXTENT_CHUNK_METADATA_BYTES;
        let chunk_count = u32::try_from(
            placement
                .payload_bytes()
                .div_ceil(u64::from(maximum_frame_bytes.checked_sub(overhead as u32)?)),
        )
        .ok()?;
        worth_store_physical_format::DurableExtentManifest::new(
            format,
            placement.record(),
            placement.extent_cell(),
            placement.payload_bytes(),
            maximum_frame_bytes,
            chunk_count,
        )
    }

    pub(in crate::physical_runtime) const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }
}

impl IntegrityAdmittedResidentExtentChunkView<'_> {
    pub(in crate::physical_runtime) fn project_chunk(
        &self,
        expected: worth_store_physical_format::ExtentChunkCoordinate,
    ) -> Result<ResidentExtentChunkProjection, ExtentChunkProjectionDenial> {
        let coordinate = self
            .scope
            .extent_chunk_coordinate()
            .ok_or(ExtentChunkProjectionDenial::ExtentIdentityMismatch)?;
        if expected.record() != coordinate.record() {
            return Err(ExtentChunkProjectionDenial::RecordIdentityMismatch);
        }
        if expected.extent_cell().extent_id() != coordinate.extent_cell().extent_id() {
            return Err(ExtentChunkProjectionDenial::ExtentIdentityMismatch);
        }
        if expected.extent_cell().generation() != coordinate.extent_cell().generation() {
            return Err(ExtentChunkProjectionDenial::ExtentGenerationMismatch);
        }
        if expected.logical_bytes() != coordinate.logical_bytes() {
            return Err(ExtentChunkProjectionDenial::LogicalLengthMismatch);
        }
        if expected.logical_offset() != coordinate.logical_offset() {
            return Err(ExtentChunkProjectionDenial::LogicalOffsetMismatch);
        }
        if expected.ordinal() != coordinate.ordinal() {
            return Err(ExtentChunkProjectionDenial::ChunkOrdinalMismatch);
        }
        let payload_start = worth_store_physical_format::DURABLE_EXTENT_FRAME_HEADER_BYTES
            + worth_store_physical_format::EXTENT_CHUNK_METADATA_BYTES;
        Ok(ResidentExtentChunkProjection {
            payload: payload_start..self.lease.len(),
            page_lsn: admitted_page_lsn(self.lease)
                .ok_or(ExtentChunkProjectionDenial::LogicalLengthMismatch)?,
        })
    }

    pub(in crate::physical_runtime) const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }
}

pub(in crate::physical_runtime) struct ResidentExtentChunkProjection {
    pub(in crate::physical_runtime) payload: std::ops::Range<usize>,
    pub(in crate::physical_runtime) page_lsn: worth_store_physical_format::PhysicalPageLsn,
}

fn admitted_page_lsn(bytes: &[u8]) -> Option<worth_store_physical_format::PhysicalPageLsn> {
    let encoded = bytes.get(36..44)?;
    Some(worth_store_physical_format::PhysicalPageLsn::new(
        u64::from_le_bytes(encoded.try_into().ok()?),
    ))
}
