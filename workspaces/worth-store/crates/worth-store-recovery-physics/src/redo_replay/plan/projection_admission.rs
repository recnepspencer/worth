use std::ops::Range;

use super::*;
use worth_store_physical_format::{
    DurableExtentManifest, DurableInlineRecordPlacement, ExtentChunkCoordinate, PhysicalPageLsn,
};
use worth_store_physical_integrity::{
    validate_extent_chunk_membership, validate_extent_manifest, validate_inline_page,
    ExtentChunkIntegrityValidation, ExtentManifestIntegrityValidation,
    InlinePageIntegrityValidation, IntegrityValidatedExtentMembership, PhysicalArtifactScope,
    PhysicalByteRange, UntrustedPhysicalArtifact,
};

pub(super) struct IntegrityAdmittedRecoveryProjection<'projection> {
    projection: &'projection PersistedPhysicalRecoveryProjection,
    inline_frames: Vec<AdmittedInlineFrame>,
    extent_chunks: Vec<AdmittedExtentChunk>,
    extent_manifests: Vec<AdmittedExtentManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AdmittedInlineFrame {
    pub frame_index: usize,
    pub page_lsn: PhysicalPageLsn,
    pub records: Vec<(DurableInlineRecordPlacement, Range<usize>)>,
}

impl IntegrityAdmittedRecoveryProjection<'_> {
    pub(super) fn into_inline_frames(
        self,
        budget: &mut RedoAllocationBudget,
    ) -> Result<Box<[AdmittedInlineFrame]>, PhysicalRedoPlanningDenial> {
        let Self {
            inline_frames,
            extent_chunks,
            extent_manifests,
            ..
        } = self;
        budget.release_vec(extent_chunks);
        budget.release_vec(extent_manifests);
        budget.into_boxed_slice(inline_frames)
    }
}

pub(super) struct AdmittedExtentChunk {
    pub frame_index: usize,
    pub coordinate: ExtentChunkCoordinate,
    pub page_lsn: PhysicalPageLsn,
    pub payload_range: Range<usize>,
}

pub(super) struct AdmittedExtentManifest {
    pub manifest: DurableExtentManifest,
    pub membership: IntegrityValidatedExtentMembership,
}

pub(super) fn admit_projection<'projection>(
    projection: &'projection PersistedPhysicalRecoveryProjection,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut RedoAllocationBudget,
) -> Result<IntegrityAdmittedRecoveryProjection<'projection>, PhysicalRedoPlanningDenial> {
    let Some(frames) = projection.frames() else {
        return invalid();
    };
    let inline_count = frames
        .iter()
        .filter(|frame| matches!(frame.subject(), PersistedPhysicalDataFrameSubject::InlinePage(_)))
        .count();
    let chunk_count = frames.len() - inline_count;
    let mut admitted = IntegrityAdmittedRecoveryProjection {
        projection,
        inline_frames: budget.reserve_exact(inline_count)?,
        extent_chunks: budget.reserve_exact(chunk_count)?,
        extent_manifests: budget.reserve_exact(projection.manifests().len())?,
    };
    for source in projection.manifests() {
        let mut placements = projection.placements().iter().filter_map(|placement| match placement {
            CurrentPhysicalRecordPlacement::Extent(value)
                if source.artifact()
                    == (worth_store_physical_format::RecordArtifactFile::ExtentArena {
                        arena: value.arena_range().arena().get(),
                    })
                    && source.coordinate().offset() == value.arena_range().offset() =>
            {
                Some(*value)
            }
            _ => None,
        });
        let Some(placement) = placements.next() else {
            return invalid();
        };
        if placements.next().is_some() {
            return invalid();
        }
        let input = UntrustedPhysicalArtifact::from_bounded_bytes(source.bytes());
        let scope = PhysicalArtifactScope::extent_manifest(
            store,
            format,
            placement,
            coordinate_range(source.coordinate())?,
        );
        let (validation, _) = validate_extent_manifest(input, scope);
        let ExtentManifestIntegrityValidation::Intact(validated) = validation else {
            return invalid();
        };
        let manifest = DurableExtentManifest::new(
            validated.record_format(),
            validated.record(),
            validated.extent_cell(),
            validated.logical_bytes(),
            validated.maximum_frame_bytes(),
            validated.chunk_count(),
            validated.alignment(),
        )
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
        admitted.extent_manifests.push(AdmittedExtentManifest {
            manifest,
            membership: validated.membership(),
        });
    }
    for (frame_index, frame) in frames.iter().enumerate() {
        match frame.subject() {
            PersistedPhysicalDataFrameSubject::InlinePage(page) => {
                admit_inline(&mut admitted, frame_index, page, store, format, budget)?;
            }
            PersistedPhysicalDataFrameSubject::ExtentChunk(coordinate) => {
                admit_chunk(&mut admitted, frame_index, coordinate, store, format)?;
            }
        }
    }
    admitted.extent_chunks.sort_unstable_by_key(|chunk| {
        (
            chunk.coordinate.record(),
            chunk.coordinate.extent_cell().extent_id().get(),
            chunk.coordinate.extent_cell().generation().get(),
            chunk.coordinate.ordinal(),
        )
    });
    Ok(admitted)
}

fn admit_inline(
    admitted: &mut IntegrityAdmittedRecoveryProjection<'_>,
    frame_index: usize,
    page: worth_store_physical_format::PageGenerationCell,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut RedoAllocationBudget,
) -> Result<(), PhysicalRedoPlanningDenial> {
    let frame = &admitted
        .projection
        .frames()
        .expect("frame-only recovery projection admitted before planning")[frame_index];
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(frame.bytes());
    let scope = PhysicalArtifactScope::inline_page(
        store,
        format,
        page,
        coordinate_range(frame.coordinate())?,
    );
    let (validation, _) = validate_inline_page(input, scope);
    let InlinePageIntegrityValidation::Intact(validated) = validation else {
        return invalid();
    };
    let count = admitted
        .projection
        .placements()
        .iter()
        .filter(|placement| {
            matches!(placement, CurrentPhysicalRecordPlacement::Inline(value) if value.page_cell() == page)
        })
        .count();
    let mut records = budget.reserve_exact(count)?;
    for placement in admitted.projection.placements() {
        let CurrentPhysicalRecordPlacement::Inline(placement) = placement else {
            continue;
        };
        if placement.page_cell() != page {
            continue;
        }
        let projected = validated
            .project_record(input, *placement)
            .map_err(|_| PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
        records.push((*placement, projected.payload_range()));
    }
    if records.len() != usize::from(validated.slot_count()) {
        return invalid();
    }
    admitted.inline_frames.push(AdmittedInlineFrame {
        frame_index,
        page_lsn: validated.page_lsn(),
        records,
    });
    Ok(())
}

fn admit_chunk(
    admitted: &mut IntegrityAdmittedRecoveryProjection<'_>,
    frame_index: usize,
    coordinate: ExtentChunkCoordinate,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
) -> Result<(), PhysicalRedoPlanningDenial> {
    let frame = &admitted
        .projection
        .frames()
        .expect("frame-only recovery projection admitted before planning")[frame_index];
    let manifest = admitted
        .extent_manifests
        .iter()
        .find(|value| {
            value.manifest.record() == coordinate.record()
                && value.manifest.extent_cell() == coordinate.extent_cell()
        })
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(frame.bytes());
    let scope = PhysicalArtifactScope::extent_chunk(
        store,
        format,
        coordinate,
        coordinate_range(frame.coordinate())?,
        manifest.membership.arena_range(),
    );
    let (validation, _) = validate_extent_chunk_membership(input, scope, manifest.membership);
    let ExtentChunkIntegrityValidation::Intact(validated) = validation else {
        return invalid();
    };
    let projected = validated
        .project_chunk(input, coordinate)
        .map_err(|_| PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    admitted.extent_chunks.push(AdmittedExtentChunk {
        frame_index,
        coordinate,
        page_lsn: projected.page_lsn(),
        payload_range: projected.payload_range(),
    });
    Ok(())
}

impl IntegrityAdmittedRecoveryProjection<'_> {
    pub fn inline_frames(&self) -> &[AdmittedInlineFrame] {
        &self.inline_frames
    }
    #[cfg(test)]
    pub fn extent_chunks(&self) -> &[AdmittedExtentChunk] {
        &self.extent_chunks
    }
    pub fn extent_chunks_for(
        &self,
        record: worth_store_physical_format::PersistedRecordIdentity,
        extent_cell: worth_store_physical_format::RecordExtentGenerationCell,
    ) -> &[AdmittedExtentChunk] {
        let target = (
            record,
            extent_cell.extent_id().get(),
            extent_cell.generation().get(),
        );
        let key = |chunk: &AdmittedExtentChunk| {
            (
                chunk.coordinate.record(),
                chunk.coordinate.extent_cell().extent_id().get(),
                chunk.coordinate.extent_cell().generation().get(),
            )
        };
        let start = self.extent_chunks.partition_point(|chunk| key(chunk) < target);
        let end = start
            + self.extent_chunks[start..].partition_point(|chunk| key(chunk) == target);
        &self.extent_chunks[start..end]
    }
    pub fn extent_manifests(&self) -> &[AdmittedExtentManifest] {
        &self.extent_manifests
    }
    pub fn frame_bytes(&self, index: usize) -> &[u8] {
        self.projection
            .frames()
            .expect("frame-only recovery projection admitted before planning")[index]
            .bytes()
    }
}

fn coordinate_range(
    coordinate: worth_store_physical_format::RecordFrameCoordinate,
) -> Result<PhysicalByteRange, PhysicalRedoPlanningDenial> {
    PhysicalByteRange::new(coordinate.offset(), u64::from(coordinate.length()))
        .map_err(|_| PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
}

fn invalid<T>() -> Result<T, PhysicalRedoPlanningDenial> {
    Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
}

#[cfg(test)]
#[path = "projection_admission/retired_inline_tests.rs"]
mod retired_inline_tests;

#[cfg(test)]
#[path = "projection_admission/tests.rs"]
mod tests;
