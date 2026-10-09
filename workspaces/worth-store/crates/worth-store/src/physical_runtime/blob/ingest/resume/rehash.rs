use sha2::{Digest, Sha256};
use worth_store_physical_format::{decode_blob_record, BlobRecordV1};

use super::super::super::verify_selected_reuse_source;
use super::super::frontier::BlobIngestProgress;
use super::{
    claims::SelectedResumeClaim,
    record_read::{read_record_into, RESUME_FRAME_BYTES},
    selection::SelectedResumeState,
    BlobResumeFailure, BlobResumeToken,
};

/// Reconstructs logical identity only from selected, authenticated C5 chunk
/// bytes. The earlier claim scan establishes order and capacity; this pass
/// rereads every chunk through its protected root before admitting a writer.
pub(super) fn rehash(
    state: &mut SelectedResumeState,
    token: BlobResumeToken,
) -> Result<(Sha256, BlobIngestProgress), BlobResumeFailure> {
    let mut logical = Sha256::new();
    let mut progress = BlobIngestProgress::new(token);
    let mut next_ordinal = 0_u64;
    for index in 0..state.claims.len() {
        let claim = state.claims[index];
        let Some((ordinal, record, digest, bytes)) = claim.chunk() else {
            continue;
        };
        if ordinal != next_ordinal {
            return Err(BlobResumeFailure::ConflictingClaims);
        }
        let used = read_record_into(
            &state.reader,
            record,
            &mut state.scratch,
            RESUME_FRAME_BYTES,
        )?;
        match decode_blob_record(&state.scratch[..used]).map_err(BlobResumeFailure::Format)? {
            BlobRecordV1::Chunk(chunk) if matches!(claim, SelectedResumeClaim::Chunk { .. }) => {
                let occurrence = chunk.occurrence();
                if occurrence.store() != state.declaration.store()
                    || occurrence.session() != state.declaration.session()
                    || occurrence.ordinal() != ordinal
                    || chunk.chunk_size() != state.declaration.chunk_size()
                    || chunk.stored_digest() != digest
                    || chunk.bytes().len() as u64 != bytes
                {
                    return Err(BlobResumeFailure::ConflictingClaims);
                }
                logical.update(chunk.bytes());
            }
            BlobRecordV1::ChunkReuseClaim(reuse)
                if matches!(claim, SelectedResumeClaim::ReusedChunk { .. }) =>
            {
                if reuse.store() != state.declaration.store()
                    || reuse.destination_session() != state.declaration.session()
                    || reuse.destination_ordinal() != ordinal
                    || reuse.scope() != state.declaration.key_scope()
                    || reuse.chunk_size() != state.declaration.chunk_size()
                    || reuse.stored_digest() != digest
                    || u64::from(reuse.chunk_length()) != bytes
                {
                    return Err(BlobResumeFailure::ConflictingClaims);
                }
                let source = super::super::super::verify_selected_reuse_source_v1(
                    &state.reader,
                    record,
                    reuse,
                    &mut state.scratch,
                )
                .map_err(BlobResumeFailure::Reuse)?;
                logical.update(source.bytes);
            }
            BlobRecordV1::ChunkReuseClaimV2(reuse)
                if matches!(claim, SelectedResumeClaim::ReusedChunk { .. }) =>
            {
                let base = reuse.claim();
                if base.store() != state.declaration.store()
                    || base.destination_session() != state.declaration.session()
                    || base.destination_ordinal() != ordinal
                    || base.scope() != state.declaration.key_scope()
                    || base.chunk_size() != state.declaration.chunk_size()
                    || base.stored_digest() != digest
                    || u64::from(base.chunk_length()) != bytes
                {
                    return Err(BlobResumeFailure::ConflictingClaims);
                }
                let source =
                    verify_selected_reuse_source(&state.reader, record, reuse, &mut state.scratch)
                        .map_err(BlobResumeFailure::Reuse)?;
                logical.update(source.bytes);
            }
            _ => return Err(BlobResumeFailure::ConflictingClaims),
        }
        progress.record_chunk(record, digest, bytes);
        state.observation.rehashed_chunks = state
            .observation
            .rehashed_chunks
            .checked_add(1)
            .ok_or(BlobResumeFailure::ConflictingClaims)?;
        state.observation.rehashed_bytes = state
            .observation
            .rehashed_bytes
            .checked_add(bytes)
            .ok_or(BlobResumeFailure::ConflictingClaims)?;
        next_ordinal += 1;
    }
    let persisted_ordinal = state
        .claims
        .iter()
        .filter_map(|claim| match *claim {
            SelectedResumeClaim::Frontier { ordinal, .. } => Some(ordinal),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    progress.restore_persisted_frontier(persisted_ordinal);
    Ok((logical, progress))
}
