use crate::{
    AuthenticatedFrameDigest, BlobChunkReachabilityProofSet, BlobChunkSecurityMetadataWitness,
    BlobGeneration, BlobObjectId, ChunkTreeRoot, LifecycleReceipt, LogicalContentDigest,
    StoredChunkDigest,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlobPlacementMovementBasis {
    object_id: BlobObjectId,
    generation: BlobGeneration,
    chunk_tree_root: ChunkTreeRoot,
    logical_content_digest: LogicalContentDigest,
    stored_digest: StoredChunkDigest,
    authenticated_frame_digest: AuthenticatedFrameDigest,
    security_metadata: BlobChunkSecurityMetadataWitness,
}

impl BlobPlacementMovementBasis {
    pub(crate) fn from_lifecycle(receipt: &LifecycleReceipt) -> Self {
        Self {
            object_id: receipt.declaration().object_id().clone(),
            generation: receipt.declaration().generation(),
            chunk_tree_root: receipt.declaration().chunk_tree_root().clone(),
            logical_content_digest: receipt.declaration().logical_content_digest().clone(),
            stored_digest: receipt.declaration().stored_chunk_digest().clone(),
            authenticated_frame_digest: receipt.declaration().authenticated_frame_digest().clone(),
            security_metadata: receipt.declaration().security_metadata(),
        }
    }
}

#[allow(dead_code)]
fn _reachability_is_the_authority(_: &BlobChunkReachabilityProofSet) {}
