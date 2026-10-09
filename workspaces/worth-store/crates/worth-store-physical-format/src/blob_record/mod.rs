//! Versioned C.11 blob payloads carried inside selected C.5 records.
//!
//! These values validate byte representation. Store-issued identities, selected
//! C.5 routing, durability, and publication remain runtime authorities.
//! The declaration digest is SHA-256 of its complete encoded v1 frame. It is
//! not the v1 chunk/tree canonical content digest, which excludes occurrence
//! claims and is computed over the respective content subframe only.

mod chunk_frame;
mod chunk_reuse_claim;
mod chunk_reuse_claim_v2;
mod dedupe_quarantine;
mod envelope;
mod generation;
mod reclaim;
mod resume_session;
mod session;
mod tree_node;
mod validation_digest;

pub use chunk_frame::{BlobChunkFrameV1, BlobChunkOccurrenceV1, DecodedBlobChunkFrameV1};
pub use chunk_reuse_claim::BlobChunkReuseClaimV1;
pub use chunk_reuse_claim_v2::BlobChunkReuseClaimV2;
pub use dedupe_quarantine::BlobDedupeQuarantineV1;
pub use envelope::{
    BlobRecordDenial, BlobRecordKind, BLOB_CHUNK_FRAME_MAX_BYTES, BLOB_CONTROL_FRAME_MAX_BYTES,
    BLOB_RECORD_HEADER_BYTES, BLOB_RECORD_VERSION, BLOB_TREE_NODE_FRAME_MAX_BYTES,
};
pub use generation::BlobGenerationPublicationV1;
pub use reclaim::{
    BlobReclaimDescriptorV1, BlobReclaimDescriptorV2, BlobReclaimDescriptorV3,
    BlobReclaimSourceBasisV1, BlobReclaimSourceKind, DropSetManifestV1, DropSetManifestV2,
    DropSetManifestV2View, DropSetManifestV3, DropSetManifestV3View, FailedIngestReclaimBasisV1,
    OriginalDropReservationRequestV1, OriginalDropReservedV1, ReleasedDropCustodyV1,
    ReleasedDropPredecessorV1, ReleasedGenerationReclaimBasisV1, MAXIMUM_DROP_SET_RECORDS,
};
pub use resume_session::{BlobAbandonmentReasonV1, BlobSessionAbandonedV1, BlobSessionFrontierV1};
pub use session::BlobSessionDeclarationV1;
pub use tree_node::{BlobTreeEntryV1, BlobTreeNodeKind, BlobTreeNodeV1, BlobTreeOccurrenceV1};
pub use validation_digest::blob_record_v1_validation_digest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobRecordV1<'bytes> {
    SessionDeclared(BlobSessionDeclarationV1),
    Chunk(DecodedBlobChunkFrameV1<'bytes>),
    TreeNode(BlobTreeNodeV1),
    GenerationPublished(BlobGenerationPublicationV1),
    SessionFrontier(BlobSessionFrontierV1),
    SessionAbandoned(BlobSessionAbandonedV1),
    DropSetManifest(DropSetManifestV1),
    DropSetManifestV2(DropSetManifestV2),
    OriginalDropReserved(OriginalDropReservedV1),
    ReclaimDescriptor(BlobReclaimDescriptorV1),
    ChunkReuseClaim(BlobChunkReuseClaimV1),
    ChunkReuseClaimV2(BlobChunkReuseClaimV2),
    DedupeQuarantine(BlobDedupeQuarantineV1),
    DropSetManifestV3(DropSetManifestV3),
    ReclaimDescriptorV2(BlobReclaimDescriptorV2),
    ReclaimDescriptorV3(BlobReclaimDescriptorV3),
}

impl BlobRecordV1<'_> {
    pub const fn kind(&self) -> BlobRecordKind {
        match self {
            Self::SessionDeclared(_) => BlobRecordKind::SessionDeclared,
            Self::Chunk(_) => BlobRecordKind::Chunk,
            Self::TreeNode(_) => BlobRecordKind::TreeNode,
            Self::GenerationPublished(_) => BlobRecordKind::GenerationPublished,
            Self::SessionFrontier(_) => BlobRecordKind::SessionFrontier,
            Self::SessionAbandoned(_) => BlobRecordKind::SessionAbandoned,
            Self::DropSetManifest(_) => BlobRecordKind::DropSetManifest,
            Self::DropSetManifestV2(_) => BlobRecordKind::DropSetManifestV2,
            Self::OriginalDropReserved(_) => BlobRecordKind::OriginalDropReserved,
            Self::ReclaimDescriptor(_) => BlobRecordKind::ReclaimDescriptor,
            Self::ChunkReuseClaim(_) => BlobRecordKind::ChunkReuseClaim,
            Self::ChunkReuseClaimV2(_) => BlobRecordKind::ChunkReuseClaimV2,
            Self::DedupeQuarantine(_) => BlobRecordKind::DedupeQuarantine,
            Self::DropSetManifestV3(_) => BlobRecordKind::DropSetManifestV3,
            Self::ReclaimDescriptorV2(_) => BlobRecordKind::ReclaimDescriptorV2,
            Self::ReclaimDescriptorV3(_) => BlobRecordKind::ReclaimDescriptorV3,
        }
    }
}

pub fn decode_blob_record(bytes: &[u8]) -> Result<BlobRecordV1<'_>, BlobRecordDenial> {
    let frame = envelope::decode(bytes)?;
    match frame.kind {
        BlobRecordKind::SessionDeclared => BlobSessionDeclarationV1::decode_payload(frame.payload)
            .map(BlobRecordV1::SessionDeclared),
        BlobRecordKind::Chunk => {
            DecodedBlobChunkFrameV1::decode_payload(frame.payload, frame.flags)
                .map(BlobRecordV1::Chunk)
        }
        BlobRecordKind::TreeNode => {
            BlobTreeNodeV1::decode_payload(frame.payload, frame.flags).map(BlobRecordV1::TreeNode)
        }
        BlobRecordKind::GenerationPublished => {
            BlobGenerationPublicationV1::decode_payload(frame.payload)
                .map(BlobRecordV1::GenerationPublished)
        }
        BlobRecordKind::SessionFrontier => {
            BlobSessionFrontierV1::decode_payload(frame.payload).map(BlobRecordV1::SessionFrontier)
        }
        BlobRecordKind::SessionAbandoned => BlobSessionAbandonedV1::decode_payload(frame.payload)
            .map(BlobRecordV1::SessionAbandoned),
        BlobRecordKind::DropSetManifest => {
            DropSetManifestV1::decode_payload(frame.payload).map(BlobRecordV1::DropSetManifest)
        }
        BlobRecordKind::DropSetManifestV2 => {
            DropSetManifestV2::decode_payload(frame.payload).map(BlobRecordV1::DropSetManifestV2)
        }
        BlobRecordKind::OriginalDropReserved => {
            OriginalDropReservedV1::decode_payload(frame.payload)
                .map(BlobRecordV1::OriginalDropReserved)
        }
        BlobRecordKind::ReclaimDescriptor => BlobReclaimDescriptorV1::decode_payload(frame.payload)
            .map(BlobRecordV1::ReclaimDescriptor),
        BlobRecordKind::ChunkReuseClaim => {
            BlobChunkReuseClaimV1::decode_payload(frame.payload, frame.flags)
                .map(BlobRecordV1::ChunkReuseClaim)
        }
        BlobRecordKind::ChunkReuseClaimV2 => {
            BlobChunkReuseClaimV2::decode_payload(frame.payload, frame.flags)
                .map(BlobRecordV1::ChunkReuseClaimV2)
        }
        BlobRecordKind::DedupeQuarantine => {
            BlobDedupeQuarantineV1::decode_payload(frame.payload, frame.flags)
                .map(BlobRecordV1::DedupeQuarantine)
        }
        BlobRecordKind::DropSetManifestV3 => {
            DropSetManifestV3::decode_payload(frame.payload).map(BlobRecordV1::DropSetManifestV3)
        }
        BlobRecordKind::ReclaimDescriptorV2 => {
            BlobReclaimDescriptorV2::decode_payload(frame.payload)
                .map(BlobRecordV1::ReclaimDescriptorV2)
        }
        BlobRecordKind::ReclaimDescriptorV3 => {
            BlobReclaimDescriptorV3::decode_payload(frame.payload)
                .map(BlobRecordV1::ReclaimDescriptorV3)
        }
    }
}

#[cfg(test)]
mod tests;
