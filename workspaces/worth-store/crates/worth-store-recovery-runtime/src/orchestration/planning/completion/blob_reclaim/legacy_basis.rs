//! Legacy failed-ingest reclaim still binds its selected session declaration
//! and terminal abandonment bytes before accepting a drop manifest.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobReclaimDescriptorV1, BlobSessionAbandonedV1, BlobSessionDeclarationV1, DropSetManifestV1,
};

pub(super) fn basis_valid(
    descriptor: BlobReclaimDescriptorV1,
    manifest: &DropSetManifestV1,
    declaration: BlobSessionDeclarationV1,
    declaration_bytes: &[u8],
    abandoned: BlobSessionAbandonedV1,
    abandoned_bytes: &[u8],
) -> bool {
    let source = manifest.source_basis();
    source.session() == declaration.session()
        && source.session() == abandoned.session()
        && declaration.store() == descriptor.store()
        && abandoned.store() == descriptor.store()
        && declaration.object() != [0; 16]
        && abandoned.declaration_record() == source.declaration_record()
        && abandoned.declaration_digest() == source.declaration_frame_sha256()
        && <[u8; 32]>::from(Sha256::digest(declaration_bytes)) == source.declaration_frame_sha256()
        && <[u8; 32]>::from(Sha256::digest(abandoned_bytes)) == source.abandoned_frame_sha256()
}
