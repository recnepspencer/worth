use sha2::{Digest, Sha256};

use crate::{ExtentChunkCoordinate, PageGenerationCell, RecordArtifactFile, RecordFrameCoordinate};

const ABSENT_PRIOR_IMAGE_DOMAIN: &[u8] = b"store.physical.data.certified-absent-prior-image.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PersistedPhysicalDataFrameSubject {
    InlinePage(PageGenerationCell),
    ExtentChunk(ExtentChunkCoordinate),
}

pub fn write_persisted_physical_data_frame_identity(
    subject: PersistedPhysicalDataFrameSubject,
    coordinate: RecordFrameCoordinate,
    target: &mut Vec<u8>,
) {
    write_subject(subject, target);
    write_artifact(coordinate.artifact(), target);
    target.extend_from_slice(&coordinate.offset().to_le_bytes());
    target.extend_from_slice(&coordinate.length().to_le_bytes());
}

pub fn certified_absent_prior_image_digest(
    subject: PersistedPhysicalDataFrameSubject,
    coordinate: RecordFrameCoordinate,
) -> [u8; 32] {
    let mut identity = Vec::with_capacity(96);
    write_persisted_physical_data_frame_identity(subject, coordinate, &mut identity);
    let mut digest = Sha256::new();
    digest.update((ABSENT_PRIOR_IMAGE_DOMAIN.len() as u64).to_le_bytes());
    digest.update(ABSENT_PRIOR_IMAGE_DOMAIN);
    digest.update((identity.len() as u64).to_le_bytes());
    digest.update(identity);
    digest.finalize().into()
}

fn write_subject(subject: PersistedPhysicalDataFrameSubject, target: &mut Vec<u8>) {
    match subject {
        PersistedPhysicalDataFrameSubject::InlinePage(page) => {
            target.push(1);
            target.extend_from_slice(&page.segment_id().get().to_le_bytes());
            target.extend_from_slice(&page.page_id().get().to_le_bytes());
            target.extend_from_slice(&page.generation().get().to_le_bytes());
        }
        PersistedPhysicalDataFrameSubject::ExtentChunk(chunk) => {
            target.push(2);
            let record = chunk.record();
            target.extend_from_slice(&record.allocation_epoch());
            target.extend_from_slice(&record.ordinal().to_le_bytes());
            target.extend_from_slice(&chunk.extent_cell().extent_id().get().to_le_bytes());
            target.extend_from_slice(&chunk.extent_cell().generation().get().to_le_bytes());
            target.extend_from_slice(&chunk.logical_bytes().to_le_bytes());
            target.extend_from_slice(&chunk.logical_offset().to_le_bytes());
            target.extend_from_slice(&chunk.ordinal().to_le_bytes());
        }
    }
}

fn write_artifact(artifact: RecordArtifactFile, target: &mut Vec<u8>) {
    match artifact {
        RecordArtifactFile::BootstrapCatalog => target.push(1),
        RecordArtifactFile::CurrentRootSelector => target.push(12),
        RecordArtifactFile::PreviousRootSelector => target.push(13),
        RecordArtifactFile::RootSelectorCandidate { role, publication } => {
            target.push(match role {
                crate::RootSelectorRole::Current => 14,
                crate::RootSelectorRole::Previous => 15,
            });
            target.extend_from_slice(&publication.to_le_bytes());
        }
        RecordArtifactFile::CatalogCandidate { publication } => {
            target.push(2);
            target.extend_from_slice(&publication.to_le_bytes());
        }
        RecordArtifactFile::RootManifest { generation } => {
            target.push(3);
            target.extend_from_slice(&generation.to_le_bytes());
        }
        RecordArtifactFile::RootRoutingBlock { generation, block } => {
            target.push(4);
            target.extend_from_slice(&generation.to_le_bytes());
            target.extend_from_slice(&block.to_le_bytes());
        }
        RecordArtifactFile::Segment {
            segment,
            generation,
        } => {
            target.push(5);
            target.extend_from_slice(&segment.to_le_bytes());
            target.extend_from_slice(&generation.to_le_bytes());
        }
        RecordArtifactFile::SegmentManifest {
            segment,
            generation,
        } => {
            target.push(6);
            target.extend_from_slice(&segment.to_le_bytes());
            target.extend_from_slice(&generation.to_le_bytes());
        }
        RecordArtifactFile::SegmentMembershipBlock { generation, block } => {
            target.push(7);
            target.extend_from_slice(&generation.to_le_bytes());
            target.extend_from_slice(&block.to_le_bytes());
        }
        RecordArtifactFile::ExtentArena { arena } => {
            target.push(16);
            target.extend_from_slice(&arena.to_le_bytes());
        }
        RecordArtifactFile::FreeSpaceManifest { generation } => {
            target.push(10);
            target.extend_from_slice(&generation.to_le_bytes());
        }
        RecordArtifactFile::FreeSpaceMembershipBlock { generation, block } => {
            target.push(11);
            target.extend_from_slice(&generation.to_le_bytes());
            target.extend_from_slice(&block.to_le_bytes());
        }
    }
}
