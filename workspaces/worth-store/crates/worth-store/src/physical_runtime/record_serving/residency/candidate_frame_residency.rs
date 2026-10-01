use worth_store_physical_format::RecordArtifactFile;

pub use self::write_evidence::CandidateFrameContractViolation;
pub(in crate::physical_runtime) use self::write_evidence::{
    CandidateFrameEffectSettlement, CandidateFrameEffectSource,
};
pub(in crate::physical_runtime::record_serving) use self::write_evidence::{
    CandidateFrameFailurePosture, CandidateFramePhysicalWrite, CandidateFrameResidencySettlement,
    CandidateFrameWriteCompletion, CandidateFrameWriteFailure,
    RecoverableCandidateFrameWriteFailure,
};
use super::super::RecordAppendDenial;

mod write_evidence;
mod write_progression;
mod writeback_progression;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::record_serving) struct CandidateFrameDeclaration {
    role: CandidateFrameRole,
    coordinate: CandidateFrameCoordinate,
    length: u32,
}

impl CandidateFrameDeclaration {
    pub(in crate::physical_runtime::record_serving) fn new(
        role: CandidateFrameRole,
        coordinate: CandidateFrameCoordinate,
        length: u32,
    ) -> Option<Self> {
        if length == 0 {
            return None;
        }
        Some(Self {
            role,
            coordinate,
            length,
        })
    }

    pub(in crate::physical_runtime::record_serving) const fn coordinate(
        self,
    ) -> CandidateFrameCoordinate {
        self.coordinate
    }

    pub(in crate::physical_runtime::record_serving) const fn role(self) -> CandidateFrameRole {
        self.role
    }

    pub(in crate::physical_runtime::record_serving) const fn length(self) -> u32 {
        self.length
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(in crate::physical_runtime::record_serving) struct CandidateFrameSet {
    frames: Vec<CandidateFrameDeclaration>,
    total_frame_bytes: u64,
}

impl CandidateFrameSet {
    pub(in crate::physical_runtime::record_serving) fn new(
        root_generation: u64,
        frames: Vec<CandidateFrameDeclaration>,
    ) -> Option<Self> {
        if root_generation == 0 || frames.is_empty() {
            return None;
        }
        let mut total_frame_bytes = 0_u64;
        let mut index = 0;
        while index < frames.len() {
            let frame = frames[index];
            if !coordinate_matches_role(frame.role, frame.coordinate.artifact()) {
                return None;
            }
            if matches!(
                frame.coordinate.artifact(),
                RecordArtifactFile::RootManifest { generation }
                    if generation != root_generation
            ) {
                return None;
            }
            total_frame_bytes = total_frame_bytes.checked_add(frame.length as u64)?;
            index += 1;
        }
        Some(Self {
            frames,
            total_frame_bytes,
        })
    }

    pub(in crate::physical_runtime::record_serving) fn frame_count(&self) -> u64 {
        self.frames.len() as u64
    }

    pub(in crate::physical_runtime::record_serving) const fn total_frame_bytes(&self) -> u64 {
        self.total_frame_bytes
    }

    pub(in crate::physical_runtime::record_serving) fn declarations(
        &self,
    ) -> &[CandidateFrameDeclaration] {
        &self.frames
    }

    fn declaration(&self, index: usize) -> Option<CandidateFrameDeclaration> {
        self.frames.get(index).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::record_serving) enum CandidateFrameRole {
    InlinePage,
    ExtentChunk,
    ExtentManifest,
    ManifestBlock,
    RootManifest,
    RootSelectorCandidate,
    CatalogCandidate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::record_serving) struct CandidateFrameCoordinate {
    artifact: RecordArtifactFile,
    offset: u64,
}

impl CandidateFrameCoordinate {
    pub(in crate::physical_runtime::record_serving) const fn new(
        artifact: RecordArtifactFile,
        offset: u64,
    ) -> Self {
        Self { artifact, offset }
    }

    pub(in crate::physical_runtime::record_serving) const fn artifact(self) -> RecordArtifactFile {
        self.artifact
    }

    pub(in crate::physical_runtime::record_serving) const fn offset(self) -> u64 {
        self.offset
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(in crate::physical_runtime::record_serving) struct CandidateFrame {
    role: CandidateFrameRole,
    coordinate: CandidateFrameCoordinate,
    bytes: Vec<u8>,
    checksum: u32,
}

impl CandidateFrame {
    pub(in crate::physical_runtime::record_serving) fn new(
        role: CandidateFrameRole,
        coordinate: CandidateFrameCoordinate,
        bytes: Vec<u8>,
    ) -> Self {
        let checksum = worth_store_physical_format::durable_artifact_checksum(&bytes);
        Self {
            role,
            coordinate,
            bytes,
            checksum,
        }
    }

    pub(in crate::physical_runtime::record_serving) const fn role(&self) -> CandidateFrameRole {
        self.role
    }

    pub(in crate::physical_runtime::record_serving) const fn coordinate(
        &self,
    ) -> CandidateFrameCoordinate {
        self.coordinate
    }

    pub(in crate::physical_runtime::record_serving) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(in crate::physical_runtime::record_serving) fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub(in crate::physical_runtime::record_serving) const fn checksum(&self) -> u32 {
        self.checksum
    }
}

pub(in crate::physical_runtime::record_serving) trait CandidateFrameResidencySession {
    fn retain(
        &mut self,
        frame: &CandidateFrame,
    ) -> Result<Box<dyn ResidentCandidateFrame>, RecordAppendDenial>;
}

pub(in crate::physical_runtime::record_serving) trait ResidentCandidateFrame {
    fn store_identity(&self) -> worth_store_physical_format::store_namespace::StableStoreIdentity;
    fn role(&self) -> CandidateFrameRole;
    fn coordinate(&self) -> CandidateFrameCoordinate;
    fn bytes(&self) -> &[u8];

    fn discard(self: Box<Self>) -> Result<(), RecordAppendDenial>;

    fn into_dirty(
        self: Box<Self>,
    ) -> Result<worth_store_buffer_pool::DirtyPhysicalFrame, RecordAppendDenial>;

    fn publish_clean(
        self: Box<Self>,
        settlement: CandidateFrameResidencySettlement,
    ) -> Result<CandidateFrameWriteCompletion, RecordAppendDenial>;
}

pub(in crate::physical_runtime::record_serving) trait CandidateFramePublicationPort {
    fn begin<'allocation>(
        &self,
        allocation: &'allocation worth_store_buffer_pool::ForegroundWriteAllocationGrant,
        candidate: &CandidateFrameSet,
    ) -> Result<Box<dyn CandidateFrameResidencySession + 'allocation>, RecordAppendDenial>;
}

impl CandidateFrameRole {
    pub(in crate::physical_runtime::record_serving) const fn is_complete_artifact(self) -> bool {
        matches!(
            self,
            Self::ManifestBlock
                | Self::RootManifest
                | Self::RootSelectorCandidate
                | Self::CatalogCandidate
        )
    }
}

pub(in crate::physical_runtime::record_serving) struct StoreCandidateFramePublicationSession<
    'allocation,
> {
    declaration: CandidateFrameSet,
    resident_frames: u64,
    resident_bytes: u64,
    next_declaration: usize,
    residency: Box<dyn CandidateFrameResidencySession + 'allocation>,
}

impl<'allocation> StoreCandidateFramePublicationSession<'allocation> {
    pub(in crate::physical_runtime::record_serving) fn begin(
        port: &(dyn CandidateFramePublicationPort + Send + Sync),
        allocation: &'allocation worth_store_buffer_pool::ForegroundWriteAllocationGrant,
        declaration: CandidateFrameSet,
    ) -> Result<Self, RecordAppendDenial> {
        let residency = port.begin(allocation, &declaration)?;
        Ok(Self {
            declaration,
            resident_frames: 0,
            resident_bytes: 0,
            next_declaration: 0,
            residency,
        })
    }

    pub(in crate::physical_runtime::record_serving) fn require_complete(
        &self,
    ) -> Result<(), CandidateFrameContractViolation> {
        if self.resident_frames != self.declaration.frame_count()
            || self.resident_bytes != self.declaration.total_frame_bytes()
        {
            return Err(CandidateFrameContractViolation::IncompleteFrameSet);
        }
        Ok(())
    }
}

const fn coordinate_matches_role(role: CandidateFrameRole, artifact: RecordArtifactFile) -> bool {
    matches!(
        (role, artifact),
        (
            CandidateFrameRole::InlinePage,
            RecordArtifactFile::Segment { .. }
        ) | (
            CandidateFrameRole::ExtentChunk | CandidateFrameRole::ExtentManifest,
            RecordArtifactFile::ExtentArena { .. }
        ) | (
            CandidateFrameRole::ManifestBlock,
            RecordArtifactFile::RootRoutingBlock { .. }
                | RecordArtifactFile::SegmentManifest { .. }
                | RecordArtifactFile::SegmentMembershipBlock { .. }
                | RecordArtifactFile::FreeSpaceManifest { .. }
                | RecordArtifactFile::FreeSpaceMembershipBlock { .. }
                | RecordArtifactFile::ReleaseCustodyHeadBlock { .. }
        ) | (
            CandidateFrameRole::RootManifest,
            RecordArtifactFile::RootManifest { .. }
        ) | (
            CandidateFrameRole::RootSelectorCandidate,
            RecordArtifactFile::RootSelectorCandidate { .. }
        ) | (
            CandidateFrameRole::CatalogCandidate,
            RecordArtifactFile::CatalogCandidate { .. }
        )
    )
}

#[cfg(test)]
#[path = "candidate_frame_residency/tests.rs"]
mod tests;
