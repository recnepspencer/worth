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

    pub(crate) const fn object_id(&self) -> &BlobObjectId {
        &self.object_id
    }

    pub(crate) const fn generation(&self) -> BlobGeneration {
        self.generation
    }

    pub(crate) const fn chunk_tree_root(&self) -> &ChunkTreeRoot {
        &self.chunk_tree_root
    }

    pub(crate) const fn logical_content_digest(&self) -> &LogicalContentDigest {
        &self.logical_content_digest
    }

    pub(crate) const fn stored_digest(&self) -> &StoredChunkDigest {
        &self.stored_digest
    }

    pub(crate) const fn authenticated_frame_digest(&self) -> &AuthenticatedFrameDigest {
        &self.authenticated_frame_digest
    }

    pub(crate) const fn security_metadata(&self) -> BlobChunkSecurityMetadataWitness {
        self.security_metadata
    }
}

#[allow(dead_code)]
fn _reachability_is_the_authority(_: &BlobChunkReachabilityProofSet) {}
