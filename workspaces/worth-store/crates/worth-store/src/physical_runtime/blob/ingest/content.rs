use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::{store_namespace::StableStoreIdentity, BlobSessionDeclarationV1};

use super::super::{BlobIngestDeclaration, BlobObjectId};
use crate::physical_runtime::PhysicalMutationDeadline;

/// Content admitted to one live attempt. The original absolute expiry remains
/// in its selected declaration/token; resuming cannot invent a new horizon.
pub(super) struct BlobIngestContent {
    object: BlobObjectId,
    chunk_size: BlobChunkSize,
    total_bytes: u64,
    scope: [u8; 32],
    deadline: PhysicalMutationDeadline,
}

impl BlobIngestContent {
    pub(super) fn from_request(declaration: &BlobIngestDeclaration) -> Self {
        Self {
            object: declaration.object(),
            chunk_size: declaration.chunk_size(),
            total_bytes: declaration.total_bytes(),
            scope: declaration.scope_fingerprint(),
            deadline: declaration.deadline(),
        }
    }

    pub(super) fn from_selected(
        declaration: BlobSessionDeclarationV1,
        store: StableStoreIdentity,
        deadline: PhysicalMutationDeadline,
    ) -> Self {
        Self {
            object: BlobObjectId::from_selected(store, declaration.object()),
            chunk_size: BlobChunkSize::from_bytes(u64::from(declaration.chunk_size()))
                .expect("decoded selected declaration has admitted fixed chunk size"),
            total_bytes: declaration.declared_bytes(),
            scope: declaration.key_scope(),
            deadline,
        }
    }

    pub(super) const fn object(&self) -> BlobObjectId {
        self.object
    }
    pub(super) const fn chunk_size(&self) -> BlobChunkSize {
        self.chunk_size
    }
    pub(super) const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
    pub(super) const fn scope_fingerprint(&self) -> [u8; 32] {
        self.scope
    }
    pub(super) const fn deadline(&self) -> PhysicalMutationDeadline {
        self.deadline
    }
}
