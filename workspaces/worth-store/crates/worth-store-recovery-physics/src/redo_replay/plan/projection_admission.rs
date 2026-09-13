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
    pub(super) fn into_inline_frames(self) -> Box<[AdmittedInlineFrame]> {
        self.inline_frames.into_boxed_slice()
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
) -> Result<IntegrityAdmittedRecoveryProjection<'projection>, PhysicalRedoPlanningDenial> {
    let mut admitted = IntegrityAdmittedRecoveryProjection {
        projection,
        inline_frames: Vec::new(),
        extent_chunks: Vec::new(),
        extent_manifests: Vec::new(),
    };
    for source in projection.manifests() {
        let placements = projection
            .placements()
            .iter()
            .filter_map(|placement| match placement {
                CurrentPhysicalRecordPlacement::Extent(value)
                    if source.artifact()
                        == worth_store_physical_format::RecordArtifactFile::ExtentManifest {
                            extent: value.extent().get(),
                            generation: value.extent_generation(),
                        } =>
                {
                    Some(*value)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        if placements.len() != 1 {
            return invalid();
        }
        let input = UntrustedPhysicalArtifact::from_bounded_bytes(source.bytes());
        let scope = PhysicalArtifactScope::extent_manifest(
            store,
            format,
            placements[0],
            range(0, source.bytes().len())?,
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
        )
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
        admitted.extent_manifests.push(AdmittedExtentManifest {
            manifest,
            membership: validated.membership(),
        });
    }
    for (frame_index, frame) in projection.frames().iter().enumerate() {
        match frame.subject() {
            PersistedPhysicalDataFrameSubject::InlinePage(page) => {
                admit_inline(&mut admitted, frame_index, page, store, format)?;
            }
            PersistedPhysicalDataFrameSubject::ExtentChunk(coordinate) => {
                admit_chunk(&mut admitted, frame_index, coordinate, store, format)?;
            }
        }
    }
    Ok(admitted)
}

fn admit_inline(
    admitted: &mut IntegrityAdmittedRecoveryProjection<'_>,
    frame_index: usize,
    page: worth_store_physical_format::PageGenerationCell,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
) -> Result<(), PhysicalRedoPlanningDenial> {
    let frame = &admitted.projection.frames()[frame_index];
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
    let mut records = Vec::new();
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
    let frame = &admitted.projection.frames()[frame_index];
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
    pub fn extent_chunks(&self) -> &[AdmittedExtentChunk] {
        &self.extent_chunks
    }
    pub fn extent_manifests(&self) -> &[AdmittedExtentManifest] {
        &self.extent_manifests
    }
    pub fn frame_bytes(&self, index: usize) -> &[u8] {
        self.projection.frames()[index].bytes()
    }
}

fn coordinate_range(
    coordinate: worth_store_physical_format::RecordFrameCoordinate,
) -> Result<PhysicalByteRange, PhysicalRedoPlanningDenial> {
    PhysicalByteRange::new(coordinate.offset(), u64::from(coordinate.length()))
        .map_err(|_| PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
}

fn range(offset: u64, length: usize) -> Result<PhysicalByteRange, PhysicalRedoPlanningDenial> {
    PhysicalByteRange::new(offset, length as u64)
        .map_err(|_| PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
}

fn invalid<T>() -> Result<T, PhysicalRedoPlanningDenial> {
    Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };
    use worth_store_physical_format::{
        encode_data_frame_page_lsn, prepare_extent_chunk, CurrentPhysicalRecordPlacement,
        DurableExtentRecordPlacement, DurableFrameKind, PersistedPhysicalDataFrameSubject,
        PersistedPhysicalRecoveryFrame, PersistedPhysicalRecoveryManifest,
        PersistedPhysicalRecoveryRootState, PersistedRecordIdentity, PhysicalExtentId,
        PhysicalGeneration, PhysicalGenerationAuthority, RecordArtifactFile, RecordFrameCoordinate,
    };

    #[test]
    fn extent_redo_admission_carries_manifest_chunk_payload_and_page_lsn() {
        let (projection, payload, coordinate) = extent_projection(false, false);
        let admitted = admit_projection(&projection, store(), format()).unwrap();
        assert_eq!(admitted.extent_manifests().len(), 1);
        assert_eq!(admitted.extent_chunks().len(), 1);
        let chunk = &admitted.extent_chunks()[0];
        assert_eq!(chunk.coordinate, coordinate);
        assert_eq!(chunk.page_lsn, PhysicalPageLsn::new(33));
        assert_eq!(
            &admitted.frame_bytes(chunk.frame_index)[chunk.payload_range.clone()],
            payload,
        );
    }

    #[test]
    fn extent_redo_rejects_corruption_and_foreign_manifest_incarnation() {
        for (corrupt_chunk, foreign_manifest) in [(true, false), (false, true)] {
            let (projection, _, _) = extent_projection(corrupt_chunk, foreign_manifest);
            assert!(matches!(
                admit_projection(&projection, store(), format()),
                Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
            ));
        }
    }

    fn extent_projection(
        corrupt_chunk: bool,
        foreign_manifest: bool,
    ) -> (
        PersistedPhysicalRecoveryProjection,
        &'static [u8],
        ExtentChunkCoordinate,
    ) {
        const PAYLOAD: &[u8] = b"extent-redo";
        let format = format();
        let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
        let record = PersistedRecordIdentity::new([4; 16], 1).unwrap();
        let extent = authority
            .record_extent_cell(PhysicalExtentId::from_raw(7).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(2).unwrap());
        let placement =
            DurableExtentRecordPlacement::new(record, extent, PAYLOAD.len() as u64).unwrap();
        let coordinate =
            ExtentChunkCoordinate::new(record, extent, PAYLOAD.len() as u64, 0, 1).unwrap();
        let mut unsealed = prepare_extent_chunk(format, coordinate, PAYLOAD.len()).unwrap();
        unsealed.payload_mut().copy_from_slice(PAYLOAD);
        let mut bytes = unsealed.seal();
        encode_data_frame_page_lsn(
            &mut bytes,
            DurableFrameKind::Extent,
            PhysicalPageLsn::new(33),
        )
        .unwrap();
        if corrupt_chunk {
            let last = bytes.len() - 1;
            bytes[last] ^= 1;
        }
        let artifact = RecordArtifactFile::Extent {
            extent: 7,
            generation: 2,
        };
        let frame = PersistedPhysicalRecoveryFrame::new(
            PersistedPhysicalDataFrameSubject::ExtentChunk(coordinate),
            RecordFrameCoordinate::new(artifact, 0, bytes.len() as u32).unwrap(),
            &bytes,
        )
        .unwrap();
        let manifest_extent = if foreign_manifest {
            authority
                .record_extent_cell(PhysicalExtentId::from_raw(7).unwrap())
                .with_extent_generation(PhysicalGeneration::from_raw(3).unwrap())
        } else {
            extent
        };
        let manifest = DurableExtentManifest::new(
            format,
            record,
            manifest_extent,
            PAYLOAD.len() as u64,
            format.page_size().bytes(),
            1,
        )
        .unwrap();
        let manifest = PersistedPhysicalRecoveryManifest::new(
            RecordArtifactFile::ExtentManifest {
                extent: 7,
                generation: 2,
            },
            &manifest.encode(format),
        )
        .unwrap();
        let root =
            PersistedPhysicalRecoveryRootState::new(4096, 1, 4, Vec::new(), None, None).unwrap();
        let projection = PersistedPhysicalRecoveryProjection::new(
            1,
            root,
            vec![record],
            vec![frame],
            vec![CurrentPhysicalRecordPlacement::Extent(placement)],
            Vec::new(),
            vec![manifest],
        )
        .unwrap();
        (projection, PAYLOAD, coordinate)
    }

    fn format() -> PhysicalRecordFormatDeclaration {
        PhysicalRecordFormatDeclaration::builder().admit().unwrap()
    }

    fn store() -> StableStoreIdentity {
        StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([0x51; 16]).unwrap(),
        )
        .published_identity()
    }
}
