use worth_store_physical_format::{BlobGenerationPublicationV1, PersistedRecordIdentity};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::blob::reachability) struct ReuseSource {
    pub publication_record: PersistedRecordIdentity,
    pub selected_chunk: PersistedRecordIdentity,
    pub source_ordinal: u64,
    pub store: [u8; 16],
    pub scope: [u8; 32],
    pub chunk_size: u32,
    pub chunk_length: u32,
    pub stored_digest: [u8; 32],
    pub witnessed_publication: Option<BlobGenerationPublicationV1>,
    pub witnessed_publication_digest: Option<[u8; 32]>,
}
