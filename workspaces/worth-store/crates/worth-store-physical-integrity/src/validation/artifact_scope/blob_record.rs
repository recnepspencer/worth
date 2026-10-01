use sha2::{Digest, Sha256};
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{BlobRecordKind, PersistedRecordIdentity};

use super::{PhysicalArtifactScope, PhysicalArtifactScopeIdentity};
use crate::localization::PhysicalByteRange;

impl PhysicalArtifactScope {
    pub const fn blob_chunk_frame(
        store: StableStoreIdentity,
        record: PersistedRecordIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::blob_record(store, record, BlobRecordKind::Chunk, range)
    }

    pub const fn blob_tree_node(
        store: StableStoreIdentity,
        record: PersistedRecordIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::blob_record(store, record, BlobRecordKind::TreeNode, range)
    }

    pub const fn blob_generation_publication(
        store: StableStoreIdentity,
        record: PersistedRecordIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::blob_record(store, record, BlobRecordKind::GenerationPublished, range)
    }

    const fn blob_record(
        store: StableStoreIdentity,
        record: PersistedRecordIdentity,
        kind: BlobRecordKind,
        range: PhysicalByteRange,
    ) -> Self {
        Self::new(
            store,
            PhysicalArtifactScopeIdentity::BlobRecord { record, kind },
            range,
        )
    }

    pub const fn blob_record_identity(self) -> Option<(PersistedRecordIdentity, BlobRecordKind)> {
        match self.identity {
            PhysicalArtifactScopeIdentity::BlobRecord { record, kind } => Some((record, kind)),
            _ => None,
        }
    }

    pub(crate) fn exact_blob_record_scope_digest(self) -> [u8; 32] {
        let (record, kind) = self
            .blob_record_identity()
            .expect("blob scope digest requires a blob record");
        let mut hash = Sha256::new();
        hash.update(b"worth-store-blob-record-integrity-scope-v1");
        hash.update(self.store.bytes());
        hash.update(record.allocation_epoch());
        hash.update(record.ordinal().to_le_bytes());
        hash.update([kind as u8]);
        hash.update(self.range.offset().to_le_bytes());
        hash.update(self.range.length().to_le_bytes());
        hash.finalize().into()
    }
}
