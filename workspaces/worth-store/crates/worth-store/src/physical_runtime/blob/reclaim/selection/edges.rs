use super::*;

pub(super) fn mark_incoming_edges(
    bytes: &[u8],
    session: [u8; 16],
    candidates: &mut [ResidueCandidate],
    frontier_prefix: &mut u64,
    occupied_attempts: &mut Vec<[u8; 16]>,
) -> Result<(), BlobReclaimFailure> {
    if !bytes.starts_with(b"WRC11BLB") {
        return Ok(());
    }
    let mut mark = |record| {
        if let Ok(index) = candidates.binary_search_by_key(&record, |candidate| candidate.record) {
            candidates[index].referenced = true;
        }
    };
    match decode_blob_record(bytes).map_err(BlobReclaimFailure::Format)? {
        BlobRecordV1::TreeNode(value) => {
            for entry in value.entries() {
                mark(entry.record());
            }
        }
        BlobRecordV1::GenerationPublished(value) => mark(value.root_record()),
        BlobRecordV1::ChunkReuseClaim(value) => mark(value.selected_chunk()),
        BlobRecordV1::ChunkReuseClaimV2(value) => mark(value.claim().selected_chunk()),
        BlobRecordV1::DedupeQuarantine(value) => {
            mark(value.source_publication());
            mark(value.source_chunk());
            mark(value.conflicting_chunk());
        }
        BlobRecordV1::SessionFrontier(value) => {
            mark(value.last_chunk_record());
            if value.session() == session {
                *frontier_prefix = (*frontier_prefix).max(value.next_chunk_ordinal());
            }
        }
        BlobRecordV1::DropSetManifest(value) => occupied_attempts.push(value.reclaim_attempt()),
        BlobRecordV1::DropSetManifestV2(value) => occupied_attempts.push(value.reclaim_attempt()),
        BlobRecordV1::DropSetManifestV3(value) => occupied_attempts.push(value.reclaim_attempt()),
        BlobRecordV1::OriginalDropReserved(value) => {
            occupied_attempts.push(value.reclaim_attempt())
        }
        BlobRecordV1::ReclaimDescriptor(value) => occupied_attempts.push(value.reclaim_attempt()),
        BlobRecordV1::ReclaimDescriptorV2(value) => occupied_attempts.push(value.reclaim_attempt()),
        BlobRecordV1::ReclaimDescriptorV3(value) => {
            occupied_attempts.push(value.base().reclaim_attempt())
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn protect_frontier_prefix(candidates: &mut [ResidueCandidate], frontier_prefix: u64) {
    for candidate in candidates {
        if candidate
            .chunk_ordinal
            .is_some_and(|ordinal| ordinal < frontier_prefix)
        {
            candidate.referenced = true;
        }
    }
}
