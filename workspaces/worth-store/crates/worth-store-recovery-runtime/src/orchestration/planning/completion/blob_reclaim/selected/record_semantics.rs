use worth_store_physical_format::{
    BlobRecordV1, BlobSessionDeclarationV1, DropSetManifestV1, PersistedRecordIdentity,
};

pub(in crate::orchestration::planning::completion::blob_reclaim) fn blob_store(
    fact: &BlobRecordV1<'_>,
) -> [u8; 16] {
    match fact {
        BlobRecordV1::SessionDeclared(value) => value.store(),
        BlobRecordV1::Chunk(value) => value.occurrence().store(),
        BlobRecordV1::TreeNode(value) => value.occurrence().store(),
        BlobRecordV1::GenerationPublished(value) => value.store(),
        BlobRecordV1::SessionFrontier(value) => value.store(),
        BlobRecordV1::SessionAbandoned(value) => value.store(),
        BlobRecordV1::DropSetManifest(value) => value.store(),
        BlobRecordV1::DropSetManifestV2(value) => value.store(),
        BlobRecordV1::DropSetManifestV3(value) => value.store(),
        BlobRecordV1::OriginalDropReserved(value) => value.store(),
        BlobRecordV1::ReclaimDescriptor(value) => value.store(),
        BlobRecordV1::ReclaimDescriptorV2(value) => value.store(),
        BlobRecordV1::ReclaimDescriptorV3(value) => value.base().store(),
        BlobRecordV1::ChunkReuseClaim(value) => value.store(),
        BlobRecordV1::ChunkReuseClaimV2(value) => value.claim().store(),
        BlobRecordV1::DedupeQuarantine(value) => value.store(),
    }
}

pub(in crate::orchestration::planning::completion::blob_reclaim) fn incoming_edge(
    fact: &BlobRecordV1<'_>,
    dropped: &[PersistedRecordIdentity],
) -> bool {
    let contains = |record| dropped.binary_search(&record).is_ok();
    match fact {
        BlobRecordV1::TreeNode(node) => node.entries().iter().any(|entry| contains(entry.record())),
        BlobRecordV1::SessionFrontier(frontier) => contains(frontier.last_chunk_record()),
        BlobRecordV1::GenerationPublished(publication) => contains(publication.root_record()),
        BlobRecordV1::ChunkReuseClaim(claim) => {
            contains(claim.selected_chunk()) || contains(claim.source_publication())
        }
        BlobRecordV1::ChunkReuseClaimV2(value) => {
            let claim = value.claim();
            contains(claim.selected_chunk()) || contains(claim.source_publication())
        }
        BlobRecordV1::DedupeQuarantine(quarantine) => {
            contains(quarantine.source_chunk())
                || contains(quarantine.source_publication())
                || contains(quarantine.conflicting_chunk())
        }
        _ => false,
    }
}

pub(super) fn publication_conflict(
    fact: &BlobRecordV1<'_>,
    declaration: BlobSessionDeclarationV1,
) -> bool {
    match fact {
        BlobRecordV1::GenerationPublished(publication) => {
            publication.session() == declaration.session()
                || publication.object() == declaration.object()
        }
        _ => false,
    }
}

pub(super) fn source_identity_conflict(
    fact: &BlobRecordV1<'_>,
    record: PersistedRecordIdentity,
    manifest: &DropSetManifestV1,
    declaration: BlobSessionDeclarationV1,
) -> bool {
    let source = manifest.source_basis();
    match fact {
        BlobRecordV1::SessionDeclared(value) => {
            (value.session() == source.session() && record != source.declaration_record())
                || (value.object() == declaration.object() && record != source.declaration_record())
        }
        BlobRecordV1::SessionAbandoned(value) => {
            (value.session() == source.session()
                || value.declaration_record() == source.declaration_record())
                && record != source.abandoned_record()
        }
        BlobRecordV1::SessionFrontier(value) => {
            (value.session() == source.session()
                || value.declaration_record() == source.declaration_record())
                && (value.session() != source.session()
                    || value.declaration_record() != source.declaration_record()
                    || value.declaration_digest() != source.declaration_frame_sha256())
        }
        _ => false,
    }
}
