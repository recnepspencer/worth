use worth_store_physical_format::{DurablePhysicalRootManifest, RecordArtifactFile};

use super::super::{
    publication::{
        append_observation::PublicationObservation, extent_publication::ExtentDataPlan,
        segment_publication::SegmentDataPlan,
    },
    residency::frame_ports::{
        CandidateFrameCoordinate, CandidateFrameDeclaration, CandidateFrameRole, CandidateFrameSet,
    },
    RecordAppendDenial,
};

pub(in crate::physical_runtime::record_serving) enum CandidateDataArtifact {
    Segment(SegmentDataPlan),
    Extent(ExtentDataPlan),
}

pub(in crate::physical_runtime::record_serving) struct PublicationPlan {
    /// Captured before payload manifests are spliced in and frame buffers move
    /// into writeback. Payload bytes are already charged by their WAL frame.
    pub(in crate::physical_runtime::record_serving) routing_metadata_bytes: Option<u64>,
    pub(in crate::physical_runtime::record_serving) arena_reservations:
        Vec<super::super::arena::ArenaReservation>,
    pub(in crate::physical_runtime::record_serving) generation: u64,
    pub(in crate::physical_runtime::record_serving) manifests:
        Vec<(worth_store_physical_format::RecordFrameCoordinate, Vec<u8>)>,
    pub(in crate::physical_runtime::record_serving) root: RecordArtifactFile,
    pub(in crate::physical_runtime::record_serving) candidate: RecordArtifactFile,
    pub(in crate::physical_runtime::record_serving) manifest: DurablePhysicalRootManifest,
    pub(in crate::physical_runtime::record_serving) root_bytes: Vec<u8>,
    pub(in crate::physical_runtime::record_serving) previous_selector_candidate: RecordArtifactFile,
    pub(in crate::physical_runtime::record_serving) previous_selector_bytes: Vec<u8>,
    pub(in crate::physical_runtime::record_serving) current_selector_candidate: RecordArtifactFile,
    pub(in crate::physical_runtime::record_serving) current_selector_bytes: Vec<u8>,
    pub(in crate::physical_runtime::record_serving) catalog_bytes: Vec<u8>,
    pub(in crate::physical_runtime::record_serving) observation: PublicationObservation,
}

impl PublicationPlan {
    /// Exact newly retained metadata, independent of live-record or free-run count.
    pub(in crate::physical_runtime::record_serving) fn retained_metadata_bytes(
        &self,
    ) -> Option<u64> {
        self.manifests
            .iter()
            .map(|(_, bytes)| bytes.len())
            .chain([
                self.root_bytes.len(),
                self.previous_selector_bytes.len(),
                self.current_selector_bytes.len(),
                self.catalog_bytes.len(),
            ])
            .try_fold(0_u64, |total, length| {
                total.checked_add(u64::try_from(length).ok()?)
            })
    }

    pub(in crate::physical_runtime::record_serving) fn root_candidate_frame_set(
        &self,
    ) -> Result<CandidateFrameSet, RecordAppendDenial> {
        self.routing_metadata_bytes
            .ok_or_else(frame_length_denial)?;
        let mut declarations = Vec::new();
        for (coordinate, bytes) in &self.manifests {
            push_candidate_declaration(
                &mut declarations,
                if matches!(
                    coordinate.artifact(),
                    RecordArtifactFile::ExtentArena { .. }
                ) {
                    CandidateFrameRole::ExtentManifest
                } else {
                    CandidateFrameRole::ManifestBlock
                },
                CandidateFrameCoordinate::new(coordinate.artifact(), coordinate.offset()),
                bytes.len() as u64,
            )?;
        }
        push_candidate_declaration(
            &mut declarations,
            CandidateFrameRole::RootManifest,
            CandidateFrameCoordinate::new(self.root, 0),
            self.root_bytes.len() as u64,
        )?;
        push_candidate_declaration(
            &mut declarations,
            CandidateFrameRole::RootSelectorCandidate,
            CandidateFrameCoordinate::new(self.previous_selector_candidate, 0),
            self.previous_selector_bytes.len() as u64,
        )?;
        push_candidate_declaration(
            &mut declarations,
            CandidateFrameRole::RootSelectorCandidate,
            CandidateFrameCoordinate::new(self.current_selector_candidate, 0),
            self.current_selector_bytes.len() as u64,
        )?;
        push_candidate_declaration(
            &mut declarations,
            CandidateFrameRole::CatalogCandidate,
            CandidateFrameCoordinate::new(self.candidate, 0),
            self.catalog_bytes.len() as u64,
        )?;
        CandidateFrameSet::new(self.generation, declarations).ok_or_else(frame_length_denial)
    }
}

fn push_candidate_declaration(
    declarations: &mut Vec<CandidateFrameDeclaration>,
    role: CandidateFrameRole,
    coordinate: CandidateFrameCoordinate,
    length: u64,
) -> Result<(), RecordAppendDenial> {
    let length = u32::try_from(length).map_err(|_| frame_length_denial())?;
    let declaration =
        CandidateFrameDeclaration::new(role, coordinate, length).ok_or_else(frame_length_denial)?;
    declarations
        .try_reserve(1)
        .map_err(|_| allocation_denial())?;
    declarations.push(declaration);
    Ok(())
}

fn frame_length_denial() -> RecordAppendDenial {
    RecordAppendDenial::from_residency(
        worth_store_buffer_pool::PhysicalResidencyDenial::FrameLengthMismatch,
    )
}

fn allocation_denial() -> RecordAppendDenial {
    RecordAppendDenial::from_residency(
        worth_store_buffer_pool::PhysicalResidencyDenial::AllocationFailed,
    )
}
