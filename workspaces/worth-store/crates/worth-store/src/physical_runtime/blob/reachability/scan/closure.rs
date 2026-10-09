use worth_store_physical_format::{BlobTreeNodeKind, PersistedRecordIdentity};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::blob::reachability) struct TreeLink {
    pub record: PersistedRecordIdentity,
    pub digest: [u8; 32],
    pub covered_bytes: u64,
}

#[derive(Clone, PartialEq, Eq)]
pub(in crate::physical_runtime::blob::reachability) enum ClosureFact {
    Publication {
        store: [u8; 16],
        session: [u8; 16],
        root: PersistedRecordIdentity,
        root_digest: [u8; 32],
        frame_digest: [u8; 32],
        total_bytes: u64,
        chunk_size: u32,
        scope: [u8; 32],
    },
    Tree {
        store: [u8; 16],
        session: [u8; 16],
        kind: BlobTreeNodeKind,
        level: u8,
        covered_bytes: u64,
        frame_digest: [u8; 32],
        canonical_digest: [u8; 32],
        links: Vec<TreeLink>,
    },
    Chunk {
        store: [u8; 16],
        session: [u8; 16],
        ordinal: u64,
        content_digest: [u8; 32],
        covered_bytes: u64,
        chunk_size: u32,
        scope: Option<[u8; 32]>,
    },
}
