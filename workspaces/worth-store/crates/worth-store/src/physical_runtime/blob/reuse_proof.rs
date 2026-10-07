use worth_store_physical_format::{
    BlobChunkReuseClaimV1, BlobChunkReuseClaimV2, BlobRecordDenial, DecodedBlobChunkFrameV1,
    PersistedRecordIdentity, BLOB_CHUNK_FRAME_MAX_BYTES,
};

use crate::physical_runtime::{
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordReadLimits,
};

use super::{
    verify_selected_claim_source_with_selected_chunk, verify_source_with_selected_chunk,
    BlobDedupeFailure, DedupeIndexValue,
};

/// The selected source bytes remain in the caller's charged scratch. This
/// proof never treats the derived dedupe locator as an authority substitute.
pub(in crate::physical_runtime) struct VerifiedReuseChunk<'scratch> {
    pub bytes: &'scratch [u8],
    pub selected_reads: u64,
    pub physical_work_count: u64,
    pub source_publication: worth_store_physical_format::BlobGenerationPublicationV1,
    pub source_publication_frame_sha256: [u8; 32],
}

pub(in crate::physical_runtime) fn verify_selected_reuse_source<'scratch>(
    reader: &PhysicalRecordReader,
    claim_record: PersistedRecordIdentity,
    claim: BlobChunkReuseClaimV2,
    scratch: &'scratch mut [u8],
) -> Result<VerifiedReuseChunk<'scratch>, BlobDedupeFailure> {
    if claim.claim().source_publication() == claim_record
        || claim.claim().selected_chunk() == claim_record
    {
        return Err(BlobDedupeFailure::SourceTreeDamaged);
    }
    verify_reuse_claim_source(reader, claim.claim(), scratch, Some(claim))
}

pub(in crate::physical_runtime) fn verify_selected_reuse_source_v1<'scratch>(
    reader: &PhysicalRecordReader,
    claim_record: PersistedRecordIdentity,
    claim: BlobChunkReuseClaimV1,
    scratch: &'scratch mut [u8],
) -> Result<VerifiedReuseChunk<'scratch>, BlobDedupeFailure> {
    if claim.source_publication() == claim_record || claim.selected_chunk() == claim_record {
        return Err(BlobDedupeFailure::SourceTreeDamaged);
    }
    verify_reuse_claim_source(reader, claim, scratch, None)
}

/// Source proof at classified-append preparation, before the destination
/// claim has a RecordId. The caller must also bind the selected root through
/// mutation rebase so this proof cannot race a source retirement.
pub(in crate::physical_runtime) fn verify_source_for_new_reuse_claim<'scratch>(
    reader: &PhysicalRecordReader,
    claim: BlobChunkReuseClaimV2,
    scratch: &'scratch mut [u8],
) -> Result<VerifiedReuseChunk<'scratch>, BlobDedupeFailure> {
    verify_reuse_claim_source(reader, claim.claim(), scratch, None).and_then(|verified| {
        if verified.source_publication != claim.source_publication()
            || verified.source_publication_frame_sha256 != claim.source_publication_frame_sha256()
        {
            return Err(BlobDedupeFailure::SourceTreeDamaged);
        }
        Ok(verified)
    })
}

fn verify_reuse_claim_source<'scratch>(
    reader: &PhysicalRecordReader,
    claim: BlobChunkReuseClaimV1,
    scratch: &'scratch mut [u8],
    selected_claim: Option<BlobChunkReuseClaimV2>,
) -> Result<VerifiedReuseChunk<'scratch>, BlobDedupeFailure> {
    if claim.store() != reader.store_identity().bytes()
        || claim.source_publication() == claim.selected_chunk()
    {
        return Err(BlobDedupeFailure::SourceTreeDamaged);
    }
    // A valid admitted 256 KiB chunk frame fits well below this fixed read
    // window. Do not let a larger caller buffer widen selected proof work.
    let maximum = scratch
        .len()
        .min(512 * 1024)
        .min(BLOB_CHUNK_FRAME_MAX_BYTES);
    let maximum = u32::try_from(maximum)
        .ok()
        .and_then(RecordByteLimit::new)
        .ok_or(BlobDedupeFailure::ScratchUnavailable)?;
    let mut opened = reader
        .open(
            PhysicalRecordId::from_persisted(claim.selected_chunk()),
            RecordReadLimits::new(maximum),
        )
        .map_err(BlobDedupeFailure::SourceRead)?;
    let mut used = 0;
    loop {
        let count = opened
            .read_next(&mut scratch[used..maximum.get() as usize])
            .map_err(BlobDedupeFailure::SourceStream)?;
        if count == 0 {
            break;
        }
        used += count;
        if used == maximum.get() as usize {
            let mut excess = [0_u8; 1];
            if opened
                .read_next(&mut excess)
                .map_err(BlobDedupeFailure::SourceStream)?
                != 0
            {
                return Err(BlobDedupeFailure::SourceFormat(
                    BlobRecordDenial::FrameTooLarge,
                ));
            }
            break;
        }
    }
    let source_work = opened.observation().physical_work_count();
    let chunk = DecodedBlobChunkFrameV1::decode(&scratch[..used])
        .map_err(BlobDedupeFailure::SourceFormat)?;
    if chunk.chunk_size() != claim.chunk_size()
        || chunk.stored_digest() != claim.stored_digest()
        || chunk.bytes().len() != claim.chunk_length() as usize
    {
        return Err(BlobDedupeFailure::SourceTreeDamaged);
    }
    let locator = DedupeIndexValue::new(
        claim.source_publication(),
        claim.source_ordinal(),
        claim.selected_chunk(),
    );
    let verified = if let Some(claim) = selected_claim {
        verify_selected_claim_source_with_selected_chunk(reader, claim, &chunk)?
    } else {
        verify_source_with_selected_chunk(
            reader,
            locator,
            claim.scope(),
            claim.stored_digest(),
            &chunk,
            claim.chunk_size(),
        )?
    };
    Ok(VerifiedReuseChunk {
        bytes: chunk.bytes(),
        selected_reads: verified.selected_reads + 1,
        physical_work_count: verified.physical_work_count.saturating_add(source_work),
        source_publication: verified.source_publication,
        source_publication_frame_sha256: verified.source_publication_frame_sha256,
    })
}
