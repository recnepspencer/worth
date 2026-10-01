use sha2::Digest;
use worth_store_physical_format::{
    BlobChunkReuseClaimV2, BlobGenerationPublicationV1, BlobRecordDenial, BlobTreeNodeKind,
    BlobTreeNodeV1, DecodedBlobChunkFrameV1, PersistedRecordIdentity, BLOB_CHUNK_FRAME_MAX_BYTES,
    BLOB_TREE_NODE_FRAME_MAX_BYTES,
};

use crate::physical_runtime::{
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordReadDenial, RecordReadLimits,
};

use super::key::DedupeIndexValue;
use super::BlobDedupeFailure;

const PUBLICATION_FRAME_BYTES: usize = 48 + 188;
const MAX_TREE_DEPTH: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct VerifiedDedupeSource {
    pub(in crate::physical_runtime::blob) publication: PersistedRecordIdentity,
    pub(in crate::physical_runtime::blob) source_ordinal: u64,
    pub(in crate::physical_runtime::blob) chunk: PersistedRecordIdentity,
    pub(in crate::physical_runtime::blob) stored_digest: [u8; 32],
    pub(in crate::physical_runtime::blob) chunk_length: u32,
    pub(in crate::physical_runtime::blob) selected_reads: u64,
    pub(in crate::physical_runtime::blob) physical_work_count: u64,
    pub(in crate::physical_runtime::blob) source_publication: BlobGenerationPublicationV1,
    pub(in crate::physical_runtime::blob) source_publication_frame_sha256: [u8; 32],
}

pub(in crate::physical_runtime) fn verify_source(
    reader: &PhysicalRecordReader,
    locator: DedupeIndexValue,
    scope: [u8; 32],
    expected_digest: [u8; 32],
    supplied: &[u8],
    chunk_size: u32,
) -> Result<VerifiedDedupeSource, BlobDedupeFailure> {
    let (chunk_bytes, chunk_work) =
        read_selected(reader, locator.chunk(), BLOB_CHUNK_FRAME_MAX_BYTES)?;
    let chunk =
        DecodedBlobChunkFrameV1::decode(&chunk_bytes).map_err(BlobDedupeFailure::SourceFormat)?;
    let mut verified = verify_source_with_selected_chunk(
        reader,
        locator,
        scope,
        expected_digest,
        &chunk,
        chunk_size,
    )?;
    if chunk.bytes() != supplied {
        return Err(BlobDedupeFailure::DigestCollisionDenied {
            scope,
            digest: expected_digest,
            source_publication: locator.publication(),
            source_ordinal: locator.source_ordinal(),
            source_chunk: locator.chunk(),
        });
    }
    verified.selected_reads += 1;
    verified.physical_work_count = verified.physical_work_count.saturating_add(chunk_work);
    Ok(verified)
}

/// Verifies source publication, exact tree edge and original occurrence for
/// a chunk that the caller already read through this protected C.5 reader.
pub(in crate::physical_runtime) fn verify_source_with_selected_chunk(
    reader: &PhysicalRecordReader,
    locator: DedupeIndexValue,
    scope: [u8; 32],
    expected_digest: [u8; 32],
    chunk: &DecodedBlobChunkFrameV1<'_>,
    chunk_size: u32,
) -> Result<VerifiedDedupeSource, BlobDedupeFailure> {
    verify_source_with_selected_chunk_policy(
        reader,
        locator,
        scope,
        expected_digest,
        chunk,
        chunk_size,
        None,
    )
}

/// Only an already selected reuse claim may use historical release custody.
/// A new dedupe hit must still prove a currently selected source publication.
pub(in crate::physical_runtime) fn verify_selected_claim_source_with_selected_chunk(
    reader: &PhysicalRecordReader,
    claim: BlobChunkReuseClaimV2,
    chunk: &DecodedBlobChunkFrameV1<'_>,
) -> Result<VerifiedDedupeSource, BlobDedupeFailure> {
    let base = claim.claim();
    verify_source_with_selected_chunk_policy(
        reader,
        DedupeIndexValue::new(
            base.source_publication(),
            base.source_ordinal(),
            base.selected_chunk(),
        ),
        base.scope(),
        base.stored_digest(),
        chunk,
        base.chunk_size(),
        Some(claim),
    )
}

fn verify_source_with_selected_chunk_policy(
    reader: &PhysicalRecordReader,
    locator: DedupeIndexValue,
    scope: [u8; 32],
    expected_digest: [u8; 32],
    chunk: &DecodedBlobChunkFrameV1<'_>,
    chunk_size: u32,
    historical_claim: Option<BlobChunkReuseClaimV2>,
) -> Result<VerifiedDedupeSource, BlobDedupeFailure> {
    let (source, source_hash, publication_work, provenance_reads) =
        match read_selected(reader, locator.publication(), PUBLICATION_FRAME_BYTES) {
            Ok((source_bytes, work)) => {
                let source = BlobGenerationPublicationV1::decode(&source_bytes)
                    .map_err(BlobDedupeFailure::SourceFormat)?;
                let hash = sha2::Sha256::digest(&source_bytes).into();
                if historical_claim.is_some_and(|claim| {
                    claim.source_publication() != source
                        || claim.source_publication_frame_sha256() != hash
                }) {
                    return Err(BlobDedupeFailure::SourceTreeDamaged);
                }
                (source, hash, work, 1)
            }
            Err(BlobDedupeFailure::SourceRead(error))
                if historical_claim.is_some()
                    && error.denial() == RecordReadDenial::RecordNotFound =>
            {
                let claim = historical_claim.expect("checked above");
                (
                    claim.source_publication(),
                    claim.source_publication_frame_sha256(),
                    0,
                    0,
                )
            }
            Err(error) => return Err(error),
        };
    if source.store() != reader.store_identity().bytes()
        || source.key_scope() != scope
        || source.chunk_size() != chunk_size
        || source.total_bytes() == 0
    {
        return Err(BlobDedupeFailure::SourceScopeMismatch);
    }
    let offset = locator
        .source_ordinal()
        .checked_mul(u64::from(chunk_size))
        .filter(|start| *start < source.total_bytes())
        .ok_or(BlobDedupeFailure::SourceTreeDamaged)?;
    let expected_length = (source.total_bytes() - offset).min(u64::from(chunk_size));
    let (node_reads, node_work) = verify_source_edge(
        reader,
        locator,
        source,
        offset,
        expected_digest,
        expected_length,
    )?;
    if chunk.occurrence().store() != source.store()
        || chunk.occurrence().session() != source.session()
        || chunk.occurrence().ordinal() != locator.source_ordinal()
        || chunk.chunk_size() != chunk_size
        || chunk.stored_digest() != expected_digest
        || chunk.bytes().len() as u64 != expected_length
    {
        return Err(BlobDedupeFailure::SourceTreeDamaged);
    }
    Ok(VerifiedDedupeSource {
        publication: locator.publication(),
        source_ordinal: locator.source_ordinal(),
        chunk: locator.chunk(),
        stored_digest: expected_digest,
        chunk_length: u32::try_from(expected_length).expect("admitted chunk fits u32"),
        selected_reads: node_reads + provenance_reads,
        physical_work_count: publication_work.saturating_add(node_work),
        source_publication: source,
        source_publication_frame_sha256: source_hash,
    })
}

fn verify_source_edge(
    reader: &PhysicalRecordReader,
    locator: DedupeIndexValue,
    source: BlobGenerationPublicationV1,
    mut offset: u64,
    expected_chunk_digest: [u8; 32],
    expected_chunk_length: u64,
) -> Result<(u64, u64), BlobDedupeFailure> {
    let mut record = source.root_record();
    let mut expected_digest = source.root_digest();
    let mut expected_bytes = source.total_bytes();
    let mut expected_level = None;
    let mut physical_work = 0_u64;
    for depth in 0..MAX_TREE_DEPTH {
        let (frame, work) = read_selected(reader, record, BLOB_TREE_NODE_FRAME_MAX_BYTES)?;
        physical_work = physical_work.saturating_add(work);
        let node = BlobTreeNodeV1::decode(&frame).map_err(BlobDedupeFailure::SourceFormat)?;
        let selected_digest = if depth == 0 {
            node.frame_digest()
        } else {
            node.canonical_digest()
        };
        if selected_digest != expected_digest
            || node.covered_bytes() != expected_bytes
            || node.occurrence().store() != source.store()
            || node.occurrence().session() != source.session()
            || expected_level.is_some_and(|level| node.occurrence().level() != level)
        {
            return Err(BlobDedupeFailure::SourceTreeDamaged);
        }
        let mut selected = None;
        for edge in node.entries() {
            if offset < edge.covered_bytes() {
                selected = Some(*edge);
                break;
            }
            offset -= edge.covered_bytes();
        }
        let edge = selected.ok_or(BlobDedupeFailure::SourceTreeDamaged)?;
        if node.occurrence().kind() == BlobTreeNodeKind::Leaf {
            if offset != 0
                || edge.record() != locator.chunk()
                || edge.digest() != expected_chunk_digest
                || edge.covered_bytes() != expected_chunk_length
            {
                return Err(BlobDedupeFailure::SourceTreeDamaged);
            }
            return Ok((depth as u64 + 1, physical_work));
        }
        expected_level = Some(
            node.occurrence()
                .level()
                .checked_sub(1)
                .ok_or(BlobDedupeFailure::SourceTreeDamaged)?,
        );
        record = edge.record();
        expected_digest = edge.digest();
        expected_bytes = edge.covered_bytes();
    }
    Err(BlobDedupeFailure::SourceTreeDamaged)
}

pub(super) fn read_selected(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
    maximum: usize,
) -> Result<(Vec<u8>, u64), BlobDedupeFailure> {
    // The admitted chunk rule tops out at 256 KiB and a tree node at 4096
    // entries. Neither can require a frame above the charged scratch ceiling.
    let maximum = maximum.min(512 * 1024);
    let limit = RecordByteLimit::new(u32::try_from(maximum).expect("fixed maximum fits u32"))
        .expect("fixed maximum is nonzero");
    let mut opened = reader
        .open(
            PhysicalRecordId::from_persisted(record),
            RecordReadLimits::new(limit),
        )
        .map_err(BlobDedupeFailure::SourceRead)?;
    let mut bytes = Vec::new();
    // All admitted chunk and tree frames fit this scratch ceiling. Avoid
    // reserving the unrelated 1 MiB read limit for every candidate lookup.
    bytes
        .try_reserve_exact(maximum)
        .map_err(|_| BlobDedupeFailure::ScratchUnavailable)?;
    let mut scratch = [0_u8; 8192];
    loop {
        let count = opened
            .read_next(&mut scratch)
            .map_err(BlobDedupeFailure::SourceStream)?;
        if count == 0 {
            return Ok((bytes, opened.observation().physical_work_count()));
        }
        if bytes.len() + count > maximum {
            return Err(BlobDedupeFailure::SourceFormat(
                BlobRecordDenial::FrameTooLarge,
            ));
        }
        bytes.extend_from_slice(&scratch[..count]);
    }
}
