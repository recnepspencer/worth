use worth_store_physical_format::{BlobRecordV1, DropSetManifestV1, PersistedRecordIdentity};

pub(super) fn selected_payload_present(
    record: PersistedRecordIdentity,
    manifest: &DropSetManifestV1,
) -> bool {
    manifest.dropped().binary_search(&record).is_ok()
}

pub(super) fn conflicts_with_manifest(
    fact: &BlobRecordV1<'_>,
    record: PersistedRecordIdentity,
    manifest: &DropSetManifestV1,
    manifest_record: PersistedRecordIdentity,
) -> bool {
    match fact {
        BlobRecordV1::ReclaimDescriptor(value) => {
            value.manifest_record() == manifest_record
                || value.reclaim_attempt() == manifest.reclaim_attempt()
        }
        BlobRecordV1::DropSetManifest(value) => {
            record != manifest_record && value.reclaim_attempt() == manifest.reclaim_attempt()
        }
        BlobRecordV1::DropSetManifestV2(value) => {
            record != manifest_record && value.reclaim_attempt() == manifest.reclaim_attempt()
        }
        BlobRecordV1::TreeNode(value) => value
            .entries()
            .iter()
            .any(|entry| entry.record() == manifest_record),
        BlobRecordV1::GenerationPublished(value) => value.root_record() == manifest_record,
        BlobRecordV1::SessionFrontier(value) => value.last_chunk_record() == manifest_record,
        _ => false,
    }
}

pub(super) fn conflicts_with_metadata_reference(
    fact: &BlobRecordV1<'_>,
    target: PersistedRecordIdentity,
) -> bool {
    match fact {
        BlobRecordV1::ReclaimDescriptor(value) => value.manifest_record() == target,
        BlobRecordV1::DropSetManifest(value) => value.dropped().binary_search(&target).is_ok(),
        BlobRecordV1::DropSetManifestV2(value) => value.dropped().binary_search(&target).is_ok(),
        BlobRecordV1::OriginalDropReserved(value) => value.manifest_record() == target,
        BlobRecordV1::TreeNode(value) => {
            value.entries().iter().any(|entry| entry.record() == target)
        }
        BlobRecordV1::GenerationPublished(value) => value.root_record() == target,
        BlobRecordV1::SessionFrontier(value) => value.last_chunk_record() == target,
        _ => false,
    }
}
