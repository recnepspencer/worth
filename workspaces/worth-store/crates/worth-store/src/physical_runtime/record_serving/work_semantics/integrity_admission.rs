use std::ops::Range;

use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, DurableExtentManifest, DurableExtentRecordPlacement,
    DurableInlineRecordPlacement, ExtentChunkCoordinate, PhysicalRecordFormatDeclaration,
};
use worth_store_physical_integrity::{
    ExtentChunkProjectionDenial, InlineRecordProjectionDenial, PhysicalArtifactScope,
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField,
    PhysicalIntegrityRejection,
};

use super::super::residency::frame_loading::LoadedPhysicalFrame;
use crate::physical_runtime::integrity::{
    admit_resident_extent_chunk, admit_resident_extent_manifest, admit_resident_page,
    IntegrityAdmittedResidentPageBasis, ResidentAdmissionContext, ResidentIntegrityAdmissionDenial,
};

#[path = "integrity_admission/denial_projection.rs"]
mod denial_projection;
#[cfg(test)]
#[path = "integrity_admission/tests.rs"]
pub(in crate::physical_runtime) mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::record_serving) enum CleanInlineAdmissionDenial {
    PageIdentity,
    SlotGeneration,
    Format,
    Unavailable,
    RuntimeReleased,
    Residency(worth_store_buffer_pool::PhysicalResidencyDenial),
    Damaged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::record_serving) enum CleanExtentAdmissionDenial {
    ExtentMembership,
    Format,
    Unavailable,
    RuntimeReleased,
    Residency(worth_store_buffer_pool::PhysicalResidencyDenial),
    Damaged,
}

pub(in crate::physical_runtime::record_serving) struct AdmittedCleanInlineRecord {
    pub payload: Range<usize>,
    pub page_lsn: worth_store_physical_format::PhysicalPageLsn,
}

pub(in crate::physical_runtime::record_serving) struct AdmittedCleanInlinePage {
    pub page: worth_store_physical_format::PageGenerationCell,
    pub slot_count: u16,
    pub free_bytes: u32,
    pub records: Vec<AdmittedCleanInlinePageRecord>,
    pub prior_basis: IntegrityAdmittedResidentPageBasis,
}

pub(in crate::physical_runtime::record_serving) struct AdmittedCleanInlinePageRecord {
    pub record: worth_store_physical_format::PersistedRecordIdentity,
    pub slot: worth_store_physical_format::PhysicalRecordSlot,
    pub slot_generation: u64,
    pub payload_bytes: u32,
}

pub(in crate::physical_runtime::record_serving) fn admit_inline_page(
    frame: &LoadedPhysicalFrame,
    context: ResidentAdmissionContext<'_>,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    page: worth_store_physical_format::PageGenerationCell,
) -> Result<AdmittedCleanInlinePage, CleanInlineAdmissionDenial> {
    let coordinate = frame.coordinate();
    let scope = PhysicalArtifactScope::inline_page(
        store,
        format,
        page,
        coordinate_range(coordinate).map_err(|_| CleanInlineAdmissionDenial::Damaged)?,
    );
    let admitted =
        admit_resident_page(frame.lease(), scope, coordinate.artifact(), context.clone())
            .map_err(classify_inline_integrity)?;
    let projection = admitted
        .with_owner_decoder(context, |view| view.project_page())
        .map_err(classify_inline_integrity)?
        .map_err(classify_inline_projection)?;
    Ok(AdmittedCleanInlinePage {
        page: projection.page,
        slot_count: projection.slot_count,
        free_bytes: projection.free_bytes,
        records: projection
            .records
            .into_iter()
            .map(|record| AdmittedCleanInlinePageRecord {
                record: record.record,
                slot: record.slot,
                slot_generation: record.slot_generation,
                payload_bytes: record.payload_bytes,
            })
            .collect(),
        prior_basis: projection.prior_basis,
    })
}

pub(in crate::physical_runtime::record_serving) struct AdmittedCleanExtentManifest {
    pub manifest: DurableExtentManifest,
    pub membership: worth_store_physical_integrity::IntegrityValidatedExtentMembership,
}

pub(in crate::physical_runtime::record_serving) struct AdmittedCleanExtentChunk {
    pub payload: Range<usize>,
    pub page_lsn: worth_store_physical_format::PhysicalPageLsn,
}

pub(in crate::physical_runtime::record_serving) fn admit_inline_record(
    frame: &LoadedPhysicalFrame,
    context: ResidentAdmissionContext<'_>,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    placement: DurableInlineRecordPlacement,
) -> Result<AdmittedCleanInlineRecord, CleanInlineAdmissionDenial> {
    let coordinate = frame.coordinate();
    let scope = PhysicalArtifactScope::inline_page(
        store,
        format,
        placement.page_cell(),
        coordinate_range(coordinate).map_err(|_| CleanInlineAdmissionDenial::Damaged)?,
    );
    let admitted =
        admit_resident_page(frame.lease(), scope, coordinate.artifact(), context.clone())
            .map_err(classify_inline_integrity)?;
    let projection = admitted
        .with_owner_decoder(context, |view| view.project_record(placement))
        .map_err(classify_inline_integrity)?
        .map_err(classify_inline_projection)?;
    Ok(AdmittedCleanInlineRecord {
        payload: projection.payload,
        page_lsn: projection.page_lsn,
    })
}

pub(in crate::physical_runtime::record_serving) fn admit_extent_manifest(
    frame: &LoadedPhysicalFrame,
    context: ResidentAdmissionContext<'_>,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    placement: DurableExtentRecordPlacement,
) -> Result<AdmittedCleanExtentManifest, CleanExtentAdmissionDenial> {
    let scope = PhysicalArtifactScope::extent_manifest(
        store,
        format,
        placement,
        coordinate_range(frame.coordinate()).map_err(|_| CleanExtentAdmissionDenial::Damaged)?,
    );
    let admitted = admit_resident_extent_manifest(frame.lease(), scope, context.clone())
        .map_err(classify_extent_integrity)?;
    let membership = admitted.membership();
    let manifest = admitted
        .with_owner_decoder(context, |view| view.project_manifest())
        .map_err(classify_extent_integrity)?
        .ok_or(CleanExtentAdmissionDenial::Damaged)?;
    Ok(AdmittedCleanExtentManifest {
        manifest,
        membership,
    })
}

pub(in crate::physical_runtime::record_serving) fn admit_extent_chunk(
    frame: &LoadedPhysicalFrame,
    context: ResidentAdmissionContext<'_>,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    coordinate: ExtentChunkCoordinate,
    membership: worth_store_physical_integrity::IntegrityValidatedExtentMembership,
) -> Result<AdmittedCleanExtentChunk, CleanExtentAdmissionDenial> {
    let scope = PhysicalArtifactScope::extent_chunk(
        store,
        format,
        coordinate,
        coordinate_range(frame.coordinate()).map_err(|_| CleanExtentAdmissionDenial::Damaged)?,
    );
    let admitted = admit_resident_extent_chunk(frame.lease(), scope, membership, context.clone())
        .map_err(classify_extent_integrity)?;
    let projection = admitted
        .with_owner_decoder(context, |view| view.project_chunk(coordinate))
        .map_err(classify_extent_integrity)?
        .map_err(classify_extent_projection)?;
    Ok(AdmittedCleanExtentChunk {
        payload: projection.payload,
        page_lsn: projection.page_lsn,
    })
}

fn coordinate_range(
    coordinate: worth_store_physical_format::RecordFrameCoordinate,
) -> Result<PhysicalByteRange, ()> {
    PhysicalByteRange::new(coordinate.offset(), u64::from(coordinate.length())).map_err(|_| ())
}

fn classify_inline_integrity(
    denial: ResidentIntegrityAdmissionDenial,
) -> CleanInlineAdmissionDenial {
    if let Some(residency) = denial.residency_unavailability() {
        return CleanInlineAdmissionDenial::Residency(residency);
    }
    match denial {
        ResidentIntegrityAdmissionDenial::LifecycleGenerationChanged => {
            CleanInlineAdmissionDenial::RuntimeReleased
        }
        ResidentIntegrityAdmissionDenial::SourceScopeMismatch => {
            CleanInlineAdmissionDenial::PageIdentity
        }
        ResidentIntegrityAdmissionDenial::Validation(PhysicalIntegrityRejection::Damaged(
            value,
        )) if (value.cause() == PhysicalDamageCause::PhysicalGenerationMismatch
            && value.field() == Some(PhysicalFormatField::PhysicalGeneration)
            && value.blast_radius() == PhysicalBlastRadius::CompleteArtifact)
            || (value.cause() == PhysicalDamageCause::ArtifactIdentityMismatch
                && value.blast_radius() == PhysicalBlastRadius::CompleteArtifact
                && matches!(
                    value.field(),
                    Some(PhysicalFormatField::SegmentIdentity | PhysicalFormatField::PageIdentity)
                )) =>
        {
            CleanInlineAdmissionDenial::PageIdentity
        }
        ResidentIntegrityAdmissionDenial::Validation(PhysicalIntegrityRejection::Damaged(_)) => {
            CleanInlineAdmissionDenial::Damaged
        }
        ResidentIntegrityAdmissionDenial::Validation(PhysicalIntegrityRejection::Unsupported(
            _,
        ))
        | ResidentIntegrityAdmissionDenial::BootstrapUnsupportedFormat(_) => {
            CleanInlineAdmissionDenial::Format
        }
        ResidentIntegrityAdmissionDenial::Validation(
            PhysicalIntegrityRejection::Unknown(_) | PhysicalIntegrityRejection::Indeterminate(_),
        ) => CleanInlineAdmissionDenial::Unavailable,
        ResidentIntegrityAdmissionDenial::BootstrapScopeMismatch(_) => {
            CleanInlineAdmissionDenial::Format
        }
        ResidentIntegrityAdmissionDenial::SourceIncarnationMismatch
        | ResidentIntegrityAdmissionDenial::FrameGenerationChanged
        | ResidentIntegrityAdmissionDenial::RetainedRecordInvalidated
        | ResidentIntegrityAdmissionDenial::RetainedRecordChanged
        | ResidentIntegrityAdmissionDenial::Frame(_) => CleanInlineAdmissionDenial::Unavailable,
    }
}

fn classify_inline_projection(denial: InlineRecordProjectionDenial) -> CleanInlineAdmissionDenial {
    match denial {
        InlineRecordProjectionDenial::PageIdentityMismatch => {
            CleanInlineAdmissionDenial::PageIdentity
        }
        InlineRecordProjectionDenial::SlotGenerationMismatch => {
            CleanInlineAdmissionDenial::SlotGeneration
        }
        InlineRecordProjectionDenial::PayloadLengthMismatch => CleanInlineAdmissionDenial::Format,
        _ => CleanInlineAdmissionDenial::Damaged,
    }
}

fn classify_extent_integrity(
    denial: ResidentIntegrityAdmissionDenial,
) -> CleanExtentAdmissionDenial {
    if let Some(residency) = denial.residency_unavailability() {
        return CleanExtentAdmissionDenial::Residency(residency);
    }
    match denial {
        ResidentIntegrityAdmissionDenial::LifecycleGenerationChanged => {
            CleanExtentAdmissionDenial::RuntimeReleased
        }
        ResidentIntegrityAdmissionDenial::SourceScopeMismatch => {
            CleanExtentAdmissionDenial::ExtentMembership
        }
        ResidentIntegrityAdmissionDenial::Validation(PhysicalIntegrityRejection::Damaged(
            value,
        )) if value.cause() == PhysicalDamageCause::PhysicalGenerationMismatch
            || value.field() == Some(PhysicalFormatField::ExtentIdentity) =>
        {
            CleanExtentAdmissionDenial::ExtentMembership
        }
        ResidentIntegrityAdmissionDenial::Validation(PhysicalIntegrityRejection::Damaged(
            value,
        )) if value.cause() == PhysicalDamageCause::FormatMismatch => {
            CleanExtentAdmissionDenial::Format
        }
        ResidentIntegrityAdmissionDenial::Validation(PhysicalIntegrityRejection::Damaged(_)) => {
            CleanExtentAdmissionDenial::Damaged
        }
        ResidentIntegrityAdmissionDenial::Validation(PhysicalIntegrityRejection::Unsupported(
            _,
        ))
        | ResidentIntegrityAdmissionDenial::BootstrapUnsupportedFormat(_) => {
            CleanExtentAdmissionDenial::Format
        }
        ResidentIntegrityAdmissionDenial::Validation(
            PhysicalIntegrityRejection::Unknown(_) | PhysicalIntegrityRejection::Indeterminate(_),
        ) => CleanExtentAdmissionDenial::Unavailable,
        ResidentIntegrityAdmissionDenial::BootstrapScopeMismatch(_) => {
            CleanExtentAdmissionDenial::Format
        }
        ResidentIntegrityAdmissionDenial::SourceIncarnationMismatch
        | ResidentIntegrityAdmissionDenial::FrameGenerationChanged
        | ResidentIntegrityAdmissionDenial::RetainedRecordInvalidated
        | ResidentIntegrityAdmissionDenial::RetainedRecordChanged
        | ResidentIntegrityAdmissionDenial::Frame(_) => CleanExtentAdmissionDenial::Unavailable,
    }
}

fn classify_extent_projection(denial: ExtentChunkProjectionDenial) -> CleanExtentAdmissionDenial {
    match denial {
        ExtentChunkProjectionDenial::ExtentGenerationMismatch => {
            CleanExtentAdmissionDenial::ExtentMembership
        }
        _ => CleanExtentAdmissionDenial::Damaged,
    }
}
