use worth_store::physical_runtime::{
    PhysicalDataFrameIdentity, PhysicalDataFrameSubject, PhysicalRedoTargetClaim,
};
use worth_store_physical_format::RecordArtifactFile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IndependentRedoTargetClaim {
    pub(super) target: Vec<u8>,
    pub(super) digest: [u8; 32],
}

pub(crate) fn independent_target_claim(
    claim: PhysicalRedoTargetClaim,
) -> IndependentRedoTargetClaim {
    IndependentRedoTargetClaim {
        target: independent_target_identity(claim.target()),
        digest: claim.resulting_payload_digest(),
    }
}

fn independent_target_identity(target: PhysicalDataFrameIdentity) -> Vec<u8> {
    let coordinate = target.coordinate();
    let mut bytes = Vec::with_capacity(96);
    match target.subject() {
        PhysicalDataFrameSubject::InlinePage(page) => {
            bytes.push(target.kind() as u8);
            bytes.extend_from_slice(&page.segment_id().get().to_le_bytes());
            bytes.extend_from_slice(&page.page_id().get().to_le_bytes());
            bytes.extend_from_slice(&page.generation().get().to_le_bytes());
        }
        PhysicalDataFrameSubject::ExtentChunk(chunk) => {
            bytes.push(target.kind() as u8);
            let record = chunk.record();
            bytes.extend_from_slice(&record.allocation_epoch());
            bytes.extend_from_slice(&record.ordinal().to_le_bytes());
            bytes.extend_from_slice(&chunk.extent_cell().extent_id().get().to_le_bytes());
            bytes.extend_from_slice(&chunk.extent_cell().generation().get().to_le_bytes());
            bytes.extend_from_slice(&chunk.logical_bytes().to_le_bytes());
            bytes.extend_from_slice(&chunk.logical_offset().to_le_bytes());
            bytes.extend_from_slice(&chunk.ordinal().to_le_bytes());
        }
    }
    match coordinate.artifact() {
        RecordArtifactFile::Segment {
            segment,
            generation,
        } => {
            bytes.push(5);
            bytes.extend_from_slice(&segment.to_le_bytes());
            bytes.extend_from_slice(&generation.to_le_bytes());
        }
        RecordArtifactFile::ExtentArena { arena } => {
            bytes.push(16);
            bytes.extend_from_slice(&arena.to_le_bytes());
            bytes.extend_from_slice(&0_u64.to_le_bytes());
        }
        _ => panic!("redo targets are data artifacts only"),
    }
    bytes.extend_from_slice(&coordinate.offset().to_le_bytes());
    bytes.extend_from_slice(&coordinate.length().to_le_bytes());
    bytes
}
