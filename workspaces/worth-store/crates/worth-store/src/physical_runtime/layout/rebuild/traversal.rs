use std::num::NonZeroU64;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobRecordV1, BlobTreeNodeKind, BlobTreeNodeV1, DecodedBlobChunkFrameV1,
    PersistedRecordIdentity,
};

use crate::physical_runtime::{
    blob::{
        verify_selected_claim_source_with_selected_chunk, verify_source_with_selected_chunk,
        DedupeIndexValue,
    },
    MaintenancePhysicalAllocation, PhysicalRecordId, PhysicalRecordReader, RecordByteLimit,
    RecordReadLimits, ServingPhysicalRuntime,
};

use super::basis::{LayoutRebuildAuthority, LayoutRebuildFailure, SelectedBlobPublication};

const MAXIMUM_FRAME_BYTES: usize = 512 * 1024;
const MAXIMUM_TREE_DEPTH: u8 = 7;
const TRAVERSAL_CHARGE_BYTES: u64 = 4 * 1024 * 1024;

pub(in crate::physical_runtime) struct ValidatedChunk<'bytes> {
    pub(in crate::physical_runtime) ordinal: u64,
    pub(in crate::physical_runtime) record: PersistedRecordIdentity,
    pub(in crate::physical_runtime) digest: [u8; 32],
    pub(in crate::physical_runtime) bytes: &'bytes [u8],
    pub(in crate::physical_runtime) original_occurrence: bool,
}

/// Visits a publication's entire selected tree, never a derived leaf. The
/// callback runs only after the chunk occurrence or selected reuse edge has
/// passed the declaration, publication and source-tree checks.
pub(in crate::physical_runtime) fn traverse_publication(
    runtime: &ServingPhysicalRuntime,
    authority: &LayoutRebuildAuthority<'_>,
    selected: SelectedBlobPublication,
    on_chunk: impl FnMut(ValidatedChunk<'_>) -> Result<(), LayoutRebuildFailure>,
) -> Result<u64, LayoutRebuildFailure> {
    traverse_publication_inner(
        runtime,
        &authority.reader,
        Some(authority),
        selected,
        on_chunk,
    )
}

/// Live maintenance follows only the newly selected publication tree. A
/// reuse edge names its selected claim directly, so no store-wide claim scan
/// or blob-sized in-memory claim collection is needed on the hot path.
pub(in crate::physical_runtime) fn traverse_publication_direct(
    runtime: &ServingPhysicalRuntime,
    reader: &PhysicalRecordReader,
    selected: SelectedBlobPublication,
    on_chunk: impl FnMut(ValidatedChunk<'_>) -> Result<(), LayoutRebuildFailure>,
) -> Result<u64, LayoutRebuildFailure> {
    traverse_publication_inner(runtime, reader, None, selected, on_chunk)
}

fn traverse_publication_inner(
    runtime: &ServingPhysicalRuntime,
    reader: &PhysicalRecordReader,
    authority: Option<&LayoutRebuildAuthority<'_>>,
    selected: SelectedBlobPublication,
    mut on_chunk: impl FnMut(ValidatedChunk<'_>) -> Result<(), LayoutRebuildFailure>,
) -> Result<u64, LayoutRebuildFailure> {
    let _allocation: MaintenancePhysicalAllocation<'_> = runtime
        .physical_allocations()
        .admit_maintenance(NonZeroU64::new(TRAVERSAL_CHARGE_BYTES).expect("nonzero charge"))
        .map_err(LayoutRebuildFailure::Allocation)?;
    let publication = selected.publication;
    let mut digest = Sha256::new();
    let mut ordinal = 0;
    let mut node_indices = [0_u64; MAXIMUM_TREE_DEPTH as usize];
    visit_node(
        reader,
        authority,
        selected,
        publication.root_record(),
        publication.root_digest(),
        publication.total_bytes(),
        None,
        0,
        &mut ordinal,
        &mut node_indices,
        &mut digest,
        &mut on_chunk,
    )?;
    if digest.finalize().as_slice() != publication.logical_digest()
        || ordinal
            != publication
                .total_bytes()
                .div_ceil(u64::from(publication.chunk_size()))
    {
        return Err(LayoutRebuildFailure::LogicalDigestMismatch);
    }
    Ok(ordinal)
}

#[allow(clippy::too_many_arguments)]
fn visit_node(
    reader: &PhysicalRecordReader,
    authority: Option<&LayoutRebuildAuthority<'_>>,
    selected: SelectedBlobPublication,
    record: PersistedRecordIdentity,
    expected_digest: [u8; 32],
    expected_bytes: u64,
    expected_level: Option<u8>,
    depth: u8,
    ordinal: &mut u64,
    node_indices: &mut [u64; MAXIMUM_TREE_DEPTH as usize],
    logical_digest: &mut Sha256,
    on_chunk: &mut impl FnMut(ValidatedChunk<'_>) -> Result<(), LayoutRebuildFailure>,
) -> Result<(), LayoutRebuildFailure> {
    if depth >= MAXIMUM_TREE_DEPTH {
        return Err(LayoutRebuildFailure::TraversalBoundExhausted);
    }
    let frame = read_selected(reader, record)?;
    let node =
        BlobTreeNodeV1::decode(&frame).map_err(LayoutRebuildFailure::MalformedSelectedBlob)?;
    let digest = if depth == 0 {
        node.frame_digest()
    } else {
        node.canonical_digest()
    };
    let level = usize::from(node.occurrence().level());
    if level >= node_indices.len() || node.occurrence().index() != node_indices[level] {
        return Err(LayoutRebuildFailure::TreeDamaged);
    }
    node_indices[level] = node_indices[level]
        .checked_add(1)
        .ok_or(LayoutRebuildFailure::TraversalBoundExhausted)?;
    if digest != expected_digest
        || node.covered_bytes() != expected_bytes
        || node.occurrence().store() != selected.publication.store()
        || node.occurrence().session() != selected.publication.session()
        || expected_level.is_some_and(|level| node.occurrence().level() != level)
    {
        return Err(LayoutRebuildFailure::TreeDamaged);
    }
    match node.occurrence().kind() {
        BlobTreeNodeKind::Leaf => {
            for edge in node.entries() {
                visit_chunk(
                    reader,
                    authority,
                    selected,
                    *ordinal,
                    *edge,
                    logical_digest,
                    on_chunk,
                )?;
                *ordinal = ordinal
                    .checked_add(1)
                    .ok_or(LayoutRebuildFailure::TraversalBoundExhausted)?;
            }
        }
        BlobTreeNodeKind::Interior => {
            let child_level = node
                .occurrence()
                .level()
                .checked_sub(1)
                .ok_or(LayoutRebuildFailure::TreeDamaged)?;
            for edge in node.entries() {
                visit_node(
                    reader,
                    authority,
                    selected,
                    edge.record(),
                    edge.digest(),
                    edge.covered_bytes(),
                    Some(child_level),
                    depth + 1,
                    ordinal,
                    node_indices,
                    logical_digest,
                    on_chunk,
                )?;
            }
        }
    }
    Ok(())
}

fn visit_chunk(
    reader: &PhysicalRecordReader,
    authority: Option<&LayoutRebuildAuthority<'_>>,
    selected: SelectedBlobPublication,
    ordinal: u64,
    edge: worth_store_physical_format::BlobTreeEntryV1,
    logical_digest: &mut Sha256,
    on_chunk: &mut impl FnMut(ValidatedChunk<'_>) -> Result<(), LayoutRebuildFailure>,
) -> Result<(), LayoutRebuildFailure> {
    let publication = selected.publication;
    let expected_start = ordinal
        .checked_mul(u64::from(publication.chunk_size()))
        .filter(|start| *start < publication.total_bytes())
        .ok_or(LayoutRebuildFailure::ChunkDamaged)?;
    let expected_len =
        (publication.total_bytes() - expected_start).min(u64::from(publication.chunk_size()));
    let frame = read_selected(reader, edge.record())?;
    if edge.covered_bytes() != expected_len {
        return Err(LayoutRebuildFailure::ChunkDamaged);
    }
    match decode_blob_record(&frame).map_err(LayoutRebuildFailure::MalformedSelectedBlob)? {
        BlobRecordV1::Chunk(chunk) => {
            if chunk.occurrence().store() != publication.store()
                || chunk.occurrence().session() != publication.session()
                || chunk.occurrence().ordinal() != ordinal
                || chunk.chunk_size() != publication.chunk_size()
                || chunk.stored_digest() != edge.digest()
                || chunk.bytes().len() as u64 != expected_len
                || authority.is_some_and(|authority| {
                    authority
                        .reuse_claims
                        .contains_key(&(publication.session(), ordinal))
                })
            {
                return Err(LayoutRebuildFailure::ChunkDamaged);
            }
            logical_digest.update(chunk.bytes());
            on_chunk(ValidatedChunk {
                ordinal,
                record: edge.record(),
                digest: edge.digest(),
                bytes: chunk.bytes(),
                original_occurrence: true,
            })
        }
        value @ (BlobRecordV1::ChunkReuseClaim(_) | BlobRecordV1::ChunkReuseClaimV2(_)) => {
            let (claim, witness) = match value {
                BlobRecordV1::ChunkReuseClaim(claim) => (claim, None),
                BlobRecordV1::ChunkReuseClaimV2(witness) => (witness.claim(), Some(witness)),
                _ => unreachable!(),
            };
            if let Some(authority) = authority {
                let selected_claim = authority
                    .reuse_claims
                    .get(&(publication.session(), ordinal))
                    .ok_or(LayoutRebuildFailure::ReuseClaimNotSelected)?;
                if selected_claim.record != edge.record()
                    || selected_claim.claim != claim
                    || selected_claim.witness != witness
                {
                    return Err(LayoutRebuildFailure::ChunkDamaged);
                }
            }
            if claim.destination_session() != publication.session()
                || claim.destination_ordinal() != ordinal
                || claim.store() != publication.store()
                || claim.scope() != publication.key_scope()
                || claim.chunk_size() != publication.chunk_size()
                || claim.chunk_length() as u64 != expected_len
                || claim.stored_digest() != edge.digest()
            {
                return Err(LayoutRebuildFailure::ChunkDamaged);
            }
            let source_frame = read_selected(reader, claim.selected_chunk())?;
            let source_chunk = DecodedBlobChunkFrameV1::decode(&source_frame)
                .map_err(LayoutRebuildFailure::MalformedSelectedBlob)?;
            if let Some(witness) = witness {
                verify_selected_claim_source_with_selected_chunk(reader, witness, &source_chunk)
            } else {
                verify_source_with_selected_chunk(
                    reader,
                    DedupeIndexValue::new(
                        claim.source_publication(),
                        claim.source_ordinal(),
                        claim.selected_chunk(),
                    ),
                    publication.key_scope(),
                    edge.digest(),
                    &source_chunk,
                    publication.chunk_size(),
                )
            }
            .map_err(LayoutRebuildFailure::ReuseAuthority)?;
            logical_digest.update(source_chunk.bytes());
            on_chunk(ValidatedChunk {
                ordinal,
                record: claim.selected_chunk(),
                digest: edge.digest(),
                bytes: source_chunk.bytes(),
                original_occurrence: false,
            })
        }
        _ => Err(LayoutRebuildFailure::ChunkDamaged),
    }
}

fn read_selected(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
) -> Result<Vec<u8>, LayoutRebuildFailure> {
    let limit = RecordByteLimit::new(MAXIMUM_FRAME_BYTES as u32).expect("nonzero bound");
    let mut read = reader
        .open(
            PhysicalRecordId::from_persisted(record),
            RecordReadLimits::new(limit),
        )
        .map_err(LayoutRebuildFailure::AuthorityRead)?;
    let mut bytes = Vec::new();
    let mut scratch = [0_u8; 8192];
    loop {
        let count = read
            .read_next(&mut scratch)
            .map_err(LayoutRebuildFailure::AuthorityStream)?;
        if count == 0 {
            return Ok(bytes);
        }
        if bytes.len() + count > MAXIMUM_FRAME_BYTES {
            return Err(LayoutRebuildFailure::TraversalBoundExhausted);
        }
        bytes
            .try_reserve(count)
            .map_err(|_| LayoutRebuildFailure::ScratchUnavailable)?;
        bytes.extend_from_slice(&scratch[..count]);
    }
}
