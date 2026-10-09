//! One resumable session, and a V3 manifest releasing a published generation.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobGenerationPublicationV1, BlobReclaimSourceBasisV1, BlobSessionDeclarationV1,
    DropSetManifestV3, PersistedRecordIdentity, ReleasedGenerationReclaimBasisV1,
};

use super::BlobResumeToken;

pub(in crate::physical_runtime::blob::ingest) fn record(
    epoch: u8,
    ordinal: u64,
) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([epoch; 16], ordinal).unwrap()
}

pub(in crate::physical_runtime::blob::ingest) fn basis(
) -> (BlobResumeToken, BlobSessionDeclarationV1) {
    let token = BlobResumeToken {
        store: [1; 16],
        session: [2; 16],
        declaration_record: record(3, 1),
        declaration_digest: [4; 32],
        chunk_size: 64 << 10,
        total_bytes: 128 << 10,
        max_checkpoint_sequence: 9,
    };
    let declaration = BlobSessionDeclarationV1::new(
        token.store,
        token.session,
        [5; 16],
        [6; 32],
        token.chunk_size,
        token.total_bytes,
        6 << 20,
        token.max_checkpoint_sequence,
    )
    .unwrap();
    (token, declaration)
}

/// The manifest of one drop batch of `dropped` records, releasing the
/// generation `session` published for `object`.
pub(in crate::physical_runtime::blob::ingest) fn released_manifest(
    store: [u8; 16],
    session: [u8; 16],
    object: [u8; 16],
    dropped: u64,
) -> DropSetManifestV3 {
    let publication = BlobGenerationPublicationV1::new(
        store,
        session,
        object,
        1,
        record(0x61, 1),
        [4; 32],
        128 << 10,
        [5; 32],
        64 << 10,
        [6; 32],
    )
    .unwrap();
    let released = ReleasedGenerationReclaimBasisV1::new(
        publication,
        record(0x61, 2),
        Sha256::digest(publication.encode()).into(),
        [7; 32],
    )
    .unwrap();
    DropSetManifestV3::new(
        store,
        [8; 16],
        BlobReclaimSourceBasisV1::ReleasedGeneration(released),
        (2..2 + dropped)
            .map(|ordinal| record(0x61, ordinal))
            .collect(),
        9,
    )
    .unwrap()
}
