pub(crate) use {std::num::NonZeroU64};

pub(crate) use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestClaimDenial, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobResumeFailure, worth_store::physical_runtime::BlobResumeLimits, worth_store::physical_runtime::BlobResumeToken, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalRecordId, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_blob_chunks::BlobChunkSize};
pub(crate) use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1};

pub(crate) use {worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::admitted_blob_scope_for_replay_boundary, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization, worth_store_testkit_store::fixture::serving_from_open};

pub(crate) const CHUNK: usize = 64 * 1024;

pub(crate) fn selected_root(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .records()
        .unwrap()
        .protected_root()
        .root()
        .generation()
        .get()
}

pub(crate) fn selected_chunk_records(
    serving: &ServingPhysicalRuntime,
) -> Vec<(u64, PhysicalRecordId, Vec<u8>)> {
    let mut chunks = selected_blob_records(serving)
        .into_iter()
        .filter_map(|(record, bytes)| match decode_blob_record(&bytes) {
            Ok(worth_store_physical_format::BlobRecordV1::Chunk(chunk)) => {
                Some((chunk.occurrence().ordinal(), record, chunk.bytes().to_vec()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    chunks.sort_by_key(|(ordinal, _, _)| *ordinal);
    chunks
}

pub(crate) fn forged_selected_mismatches(token: BlobResumeToken) -> [BlobResumeToken; 4] {
    let original = token.encode();
    let mut forged = [token; 4];
    for (index, offset) in [64, 96, 100, 108].into_iter().enumerate() {
        let mut wire = original;
        if offset == 96 {
            wire[96..100].copy_from_slice(&(128 * 1024_u32).to_le_bytes());
        } else {
            wire[offset] ^= 0x01;
        }
        forged[index] = worth_store::physical_runtime::BlobResumeToken::decode(&wire)
            .expect("each modified token must remain syntactically valid");
    }
    forged
}

pub(crate) fn patterned_chunk(ordinal: usize) -> Vec<u8> {
    (0..CHUNK)
        .map(|index| (ordinal as u8).wrapping_mul(17).wrapping_add(index as u8))
        .collect()
}