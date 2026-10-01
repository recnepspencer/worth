use worth_store_physical_format::{
    BlobChunkReuseClaimV1, BlobChunkReuseClaimV2, BlobRecordDenial, BlobTreeEntryV1,
    BlobTreeNodeKind, BlobTreeNodeV1, DecodedBlobChunkFrameV1, PersistedRecordIdentity,
    BLOB_CHUNK_FRAME_MAX_BYTES, BLOB_TREE_NODE_FRAME_MAX_BYTES,
};

use crate::physical_runtime::{PhysicalRecordReader, RecordStreamFailureKind};

use super::{BlobReadFailure, BlobReadObservation, BlobReadSession};
use crate::physical_runtime::blob::{
    verify_selected_reuse_source, verify_selected_reuse_source_v1,
};

mod record;
use record::read_record;
mod chunk_digest;
use chunk_digest::readable_chunk_digest;

const MAX_TREE_DEPTH: usize = 7;

pub(in crate::physical_runtime::blob) struct SelectedBlobNode {
    record: PersistedRecordIdentity,
    start: u64,
    node: BlobTreeNodeV1,
}

pub(in crate::physical_runtime::blob) struct SelectedBlobChunk {
    frame: Vec<u8>,
    payload_start: usize,
    start: u64,
    end: u64,
}

#[derive(Clone, Copy)]
pub(super) struct SelectedBlobChunkEdge {
    pub(super) entry: BlobTreeEntryV1,
    pub(super) start: u64,
}

pub(super) fn validate_root(session: &mut BlobReadSession<'_>) -> Result<(), BlobReadFailure> {
    let root = session.publication.root_record();
    let node = load_tree(
        &session.reader,
        root,
        TreeNodeExpectation {
            digest: session.publication.root_digest(),
            bytes: session.publication.total_bytes(),
            store: session.publication.store(),
            session: session.publication.session(),
            level: None,
        },
        &mut session.observation,
    )?;
    session.nodes.push(SelectedBlobNode {
        record: root,
        start: 0,
        node,
    });
    session.observation.tree_nodes_loaded += 1;
    Ok(())
}

pub(super) fn read_next(
    session: &mut BlobReadSession<'_>,
    target: &mut [u8],
) -> Result<usize, BlobReadFailure> {
    if target.is_empty() || session.cursor == session.end {
        return Ok(0);
    }
    if session
        .chunk
        .as_ref()
        .is_none_or(|chunk| session.cursor < chunk.start || session.cursor >= chunk.end)
    {
        session.chunk = Some(select_chunk(session, session.cursor)?);
    }
    let chunk = session.chunk.as_ref().expect("selected chunk is retained");
    let available = (chunk.end - session.cursor)
        .min(session.end - session.cursor)
        .min(target.len() as u64) as usize;
    let from = chunk.payload_start + (session.cursor - chunk.start) as usize;
    target[..available].copy_from_slice(&chunk.frame[from..from + available]);
    session.cursor += available as u64;
    session.observation.returned_bytes += available as u64;
    Ok(available)
}

fn select_chunk(
    session: &mut BlobReadSession<'_>,
    offset: u64,
) -> Result<SelectedBlobChunk, BlobReadFailure> {
    let (entry, start) = selected_chunk_edge(session, offset)?;
    session.damaged_chunk_edge = None;
    let chunk = match load_chunk(session, entry, start) {
        Ok(chunk) => chunk,
        Err(
            error @ (BlobReadFailure::ChunkCorruption { .. }
            | BlobReadFailure::ChunkFrameCorruption { .. }),
        ) => {
            session.damaged_chunk_edge = Some(SelectedBlobChunkEdge { entry, start });
            return Err(error);
        }
        Err(error) => return Err(error),
    };
    session.observation.touched_chunks += 1;
    Ok(chunk)
}

/// Authenticate the entire selected direct chunk before issuing Store's
/// non-forgeable relocation hold. A reuse-claim record is not the chunk extent.
pub(in crate::physical_runtime::blob) fn authenticated_relocation_edge(
    session: &mut BlobReadSession<'_>,
    offset: u64,
) -> Result<(PersistedRecordIdentity, u64), BlobReadFailure> {
    let (entry, start) = selected_chunk_edge(session, offset)?;
    let chunk = load_chunk(session, entry, start)?;
    if chunk.frame.get(8) != Some(&(worth_store_physical_format::BlobRecordKind::Chunk as u8)) {
        return Err(BlobReadFailure::ReuseClaimNotRelocatable);
    }
    Ok((entry.record(), chunk.end - chunk.start))
}

/// Derives a chunk edge from the protected publication/tree without trusting
/// the chunk payload. This also lets scrub inspect a damaged selected chunk.
pub(super) fn selected_chunk_edge(
    session: &mut BlobReadSession<'_>,
    offset: u64,
) -> Result<(BlobTreeEntryV1, u64), BlobReadFailure> {
    let mut record = session.publication.root_record();
    let mut expected_digest = session.publication.root_digest();
    let mut expected_bytes = session.publication.total_bytes();
    let mut base = 0_u64;
    let mut expected_level = None;
    for depth in 0..MAX_TREE_DEPTH {
        if session
            .nodes
            .get(depth)
            .is_none_or(|cached| cached.record != record || cached.start != base)
        {
            session.nodes.truncate(depth);
            let node = load_tree(
                &session.reader,
                record,
                TreeNodeExpectation {
                    digest: expected_digest,
                    bytes: expected_bytes,
                    store: session.publication.store(),
                    session: session.publication.session(),
                    level: expected_level,
                },
                &mut session.observation,
            )?;
            session.nodes.push(SelectedBlobNode {
                record,
                start: base,
                node,
            });
            session.observation.tree_nodes_loaded += 1;
        }
        let node = &session.nodes[depth].node;
        if selected_tree_digest(node, depth == 0) != expected_digest
            || node.covered_bytes() != expected_bytes
            || expected_level.is_some_and(|level| node.occurrence().level() != level)
        {
            return Err(BlobReadFailure::TreeDamaged);
        }
        let (entry, entry_start) = select_entry(node, base, offset)?;
        if node.occurrence().kind() == BlobTreeNodeKind::Leaf {
            session.nodes.truncate(depth + 1);
            return Ok((entry, entry_start));
        }
        record = entry.record();
        expected_digest = entry.digest();
        expected_bytes = entry.covered_bytes();
        base = entry_start;
        expected_level = Some(
            node.occurrence()
                .level()
                .checked_sub(1)
                .ok_or(BlobReadFailure::TreeDamaged)?,
        );
    }
    Err(BlobReadFailure::TreeDamaged)
}

fn select_entry(
    node: &BlobTreeNodeV1,
    base: u64,
    offset: u64,
) -> Result<(BlobTreeEntryV1, u64), BlobReadFailure> {
    let mut start = base;
    for entry in node.entries() {
        let end = start
            .checked_add(entry.covered_bytes())
            .ok_or(BlobReadFailure::TreeDamaged)?;
        if offset < end {
            if offset < start {
                return Err(BlobReadFailure::TreeDamaged);
            }
            return Ok((*entry, start));
        }
        start = end;
    }
    Err(BlobReadFailure::TreeDamaged)
}

struct TreeNodeExpectation {
    digest: [u8; 32],
    bytes: u64,
    store: [u8; 16],
    session: [u8; 16],
    level: Option<u8>,
}

fn load_tree(
    scan: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
    expected: TreeNodeExpectation,
    observation: &mut BlobReadObservation,
) -> Result<BlobTreeNodeV1, BlobReadFailure> {
    let frame = read_record(
        scan,
        record,
        BLOB_TREE_NODE_FRAME_MAX_BYTES as u32,
        observation,
    )?;
    let node = BlobTreeNodeV1::decode(&frame).map_err(BlobReadFailure::Format)?;
    let occurrence = node.occurrence();
    if occurrence.store() != expected.store
        || occurrence.session() != expected.session
        || selected_tree_digest(&node, expected.level.is_none()) != expected.digest
        || node.covered_bytes() != expected.bytes
        || occurrence.level() as usize >= MAX_TREE_DEPTH
        || expected
            .level
            .is_some_and(|level| occurrence.level() != level)
    {
        return Err(BlobReadFailure::TreeDamaged);
    }
    Ok(node)
}

fn selected_tree_digest(node: &BlobTreeNodeV1, is_root: bool) -> [u8; 32] {
    if is_root {
        node.frame_digest()
    } else {
        node.canonical_digest()
    }
}

fn load_chunk(
    session: &mut BlobReadSession<'_>,
    entry: BlobTreeEntryV1,
    start: u64,
) -> Result<SelectedBlobChunk, BlobReadFailure> {
    let chunk_size = u64::from(session.publication.chunk_size());
    if start % chunk_size != 0 || entry.covered_bytes() > chunk_size {
        return Err(BlobReadFailure::TreeDamaged);
    }
    let ordinal = start / chunk_size;
    let frame = read_record(
        &session.reader,
        entry.record(),
        BLOB_CHUNK_FRAME_MAX_BYTES as u32,
        &mut session.observation,
    )
    .map_err(|failure| match failure {
        BlobReadFailure::RecordStream(error)
            if error.kind() == RecordStreamFailureKind::SelectedDataFrameChecksumDamaged =>
        {
            BlobReadFailure::ChunkOuterCorruption {
                ordinal,
                record: entry.record(),
            }
        }
        other => other,
    })?;
    if matches!(frame.get(8), Some(&11 | &15)) {
        let versioned = frame.get(8)
            == Some(&(worth_store_physical_format::BlobRecordKind::ChunkReuseClaimV2 as u8));
        let claim_v2 = if versioned {
            Some(BlobChunkReuseClaimV2::decode(&frame).map_err(BlobReadFailure::Format)?)
        } else {
            None
        };
        let claim = if let Some(value) = claim_v2 {
            value.claim()
        } else {
            BlobChunkReuseClaimV1::decode(&frame).map_err(BlobReadFailure::Format)?
        };
        if claim.store() != session.publication.store()
            || claim.destination_session() != session.publication.session()
            || claim.destination_ordinal() != ordinal
            || claim.scope() != session.publication.key_scope()
            || claim.chunk_size() != session.publication.chunk_size()
            || u64::from(claim.chunk_length()) != entry.covered_bytes()
            || claim.stored_digest() != entry.digest()
        {
            return Err(BlobReadFailure::TreeDamaged);
        }
        let mut source_frame = Vec::new();
        source_frame
            .try_reserve_exact(BLOB_CHUNK_FRAME_MAX_BYTES)
            .map_err(|_| BlobReadFailure::ScratchUnavailable)?;
        source_frame.resize(BLOB_CHUNK_FRAME_MAX_BYTES, 0);
        let source_start = source_frame.as_ptr() as usize;
        let verified = if let Some(value) = claim_v2 {
            verify_selected_reuse_source(&session.reader, entry.record(), value, &mut source_frame)
        } else {
            verify_selected_reuse_source_v1(
                &session.reader,
                entry.record(),
                claim,
                &mut source_frame,
            )
        }
        .map_err(BlobReadFailure::ReuseAuthority)?;
        let payload_start = verified.bytes.as_ptr() as usize - source_start;
        session.observation.reuse_source_selected_reads += verified.selected_reads;
        session.observation.physical_work_count += verified.physical_work_count;
        let end = start
            .checked_add(entry.covered_bytes())
            .ok_or(BlobReadFailure::TreeDamaged)?;
        return Ok(SelectedBlobChunk {
            frame: source_frame,
            payload_start,
            start,
            end,
        });
    }
    let decoded = match DecodedBlobChunkFrameV1::decode(&frame) {
        Ok(decoded) => decoded,
        Err(BlobRecordDenial::IntegrityMismatch) => {
            if let Some(observed) = readable_chunk_digest(
                &frame,
                session.publication.store(),
                session.publication.session(),
                ordinal,
            )
            .filter(|observed| *observed != entry.digest())
            {
                return Err(BlobReadFailure::ChunkCorruption {
                    ordinal,
                    record: entry.record(),
                    expected: entry.digest(),
                    observed,
                });
            }
            return Err(BlobReadFailure::ChunkFrameCorruption {
                ordinal,
                record: entry.record(),
            });
        }
        Err(
            BlobRecordDenial::Truncated
            | BlobRecordDenial::WrongMagic
            | BlobRecordDenial::InvalidFlags
            | BlobRecordDenial::LengthMismatch
            | BlobRecordDenial::FrameTooLarge
            | BlobRecordDenial::MissingOccurrenceClaim
            | BlobRecordDenial::InvalidIdentity
            | BlobRecordDenial::InvalidChunkRule
            | BlobRecordDenial::InvalidChunkLength,
        ) => {
            return Err(BlobReadFailure::ChunkFrameCorruption {
                ordinal,
                record: entry.record(),
            });
        }
        Err(denial) => return Err(BlobReadFailure::Format(denial)),
    };
    let occurrence = decoded.occurrence();
    if occurrence.store() != session.publication.store()
        || occurrence.session() != session.publication.session()
        || occurrence.ordinal() != ordinal
        || decoded.chunk_size() != session.publication.chunk_size()
        || decoded.bytes().len() as u64 != entry.covered_bytes()
    {
        return Err(BlobReadFailure::TreeDamaged);
    }
    if decoded.stored_digest() != entry.digest() {
        // The chunk validates on its own terms; the selected parent edge is
        // inconsistent. Do not blame this record or issue a chunk scrub target.
        return Err(BlobReadFailure::TreeDamaged);
    }
    let payload_start = frame.len() - decoded.bytes().len();
    let end = start
        .checked_add(entry.covered_bytes())
        .ok_or(BlobReadFailure::TreeDamaged)?;
    Ok(SelectedBlobChunk {
        frame,
        payload_start,
        start,
        end,
    })
}
