//! Consolidation-invariant canonical chunk sealing and commitment hashing
//! for retained historical bases (Store Milestone 4, Phase M4.4).

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::storage::RelationalStorageSubject;
use worth_runtime_world::facade::{
    CanonicalCursorKey, CanonicalKeyDomain, HistoricalBasisRow, HistoricalReadDenial,
};

use super::chunk_codec::{encode_chunk_image, encode_continuation_page};

pub(super) const SEMANTIC_CHUNK_DOMAIN: &[u8] = b"worth.historical.chunk.semantic.v2";
pub(super) const ROW_HISTORY_DOMAIN: &[u8] = b"worth.historical.row.history.v1";
const MERKLE_LEAF_TAG: u8 = 0;
const MERKLE_NODE_TAG: u8 = 1;
const MERKLE_EMPTY_DOMAIN: &[u8] = b"worth.historical.merkle.empty.v1";

/// Default maximum canonical rows per chunk.
pub const DEFAULT_CHUNK_ROWS: usize = 16;
/// Default inline history versions per row before spilling to continuation pages.
pub const DEFAULT_INLINE_HISTORY_LIMIT: usize = 4;
/// Maximum history versions per continuation page.
pub const DEFAULT_CONTINUATION_PAGE_VERSIONS: usize = 8;

/// Content-addressed member identity (`sha256`, `byte_len`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkMemberId {
    pub sha256: [u8; 32],
    pub length: u64,
}

/// Chunking geometry for sealing a retained historical basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoricalChunkGeometry {
    pub rows_per_chunk: usize,
    pub inline_history_limit: usize,
    pub continuation_page_versions: usize,
    pub layered_initial: bool,
}

impl Default for HistoricalChunkGeometry {
    fn default() -> Self {
        Self {
            rows_per_chunk: DEFAULT_CHUNK_ROWS,
            inline_history_limit: DEFAULT_INLINE_HISTORY_LIMIT,
            continuation_page_versions: DEFAULT_CONTINUATION_PAGE_VERSIONS,
            layered_initial: true,
        }
    }
}

/// Locator binding + consolidation-invariant semantic commitment for one chunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedChunkDescriptor {
    pub semantic_commitment: [u8; 32],
    pub min_key: CanonicalCursorKey,
    pub max_key: CanonicalCursorKey,
    pub chunk_member: ChunkMemberId,
    pub continuation_members: Vec<ChunkMemberId>,
    pub(crate) chunk_locator: Option<[u8; 64]>,
    pub(crate) continuation_locators: Vec<[u8; 64]>,
    pub(crate) continuation_manifest_member: Option<ChunkMemberId>,
    pub(crate) continuation_manifest_locator: Option<[u8; 64]>,
    pub(crate) continuation_count: u32,
}

/// Complete sealed chunk set produced from canonical rows.
#[derive(Debug, Clone)]
pub struct SealedBasisChunks {
    pub root_semantic_commitment: [u8; 32],
    pub chunks: Vec<SealedChunkDescriptor>,
    pub members: BTreeMap<ChunkMemberId, Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PhysicalChunkRow {
    pub partition_id: u32,
    pub domain: CanonicalKeyDomain,
    pub kind_id: u32,
    pub total_history_bytes: u64,
    pub history_versions_sha256: [u8; 32],
    pub logical_key: Vec<u8>,
    pub payload: Vec<u8>,
    pub inline_history: Vec<Vec<u8>>,
    pub continuation_pages: Vec<ChunkMemberId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PhysicalChunkImage {
    pub layers: Vec<Vec<PhysicalChunkRow>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContinuationPageImage {
    pub partition_id: u32,
    pub domain: CanonicalKeyDomain,
    pub logical_key: Vec<u8>,
    pub sequence: u32,
    pub versions: Vec<Vec<u8>>,
}

/// Zero-copy borrowed view of one physical chunk row directly over cached chunk bytes.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BorrowedChunkRow<'a> {
    pub partition_id: u32,
    pub domain: CanonicalKeyDomain,
    pub kind_id: u32,
    pub total_history_bytes: u64,
    pub history_versions_sha256: [u8; 32],
    pub logical_key: &'a [u8],
    pub payload: &'a [u8],
    pub inline_history_raw: &'a [u8],
    pub inline_history_count: usize,
    pub inline_history_bytes: u64,
    pub continuation_pages_raw: &'a [u8],
    pub continuation_page_count: usize,
}

/// Zero-copy borrowed view of one continuation page directly over cached page bytes.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BorrowedContinuationPage<'a> {
    pub partition_id: u32,
    pub domain: CanonicalKeyDomain,
    pub logical_key: &'a [u8],
    pub sequence: u32,
    pub versions_raw: &'a [u8],
    pub version_count: usize,
}

/// Computes the canonical digest of one row's complete ordered history versions.
pub fn compute_row_history_digest(versions: &[Vec<u8>]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(ROW_HISTORY_DOMAIN);
    for version in versions {
        hasher.update((version.len() as u32).to_le_bytes());
        hasher.update(version);
    }
    hasher.finalize().into()
}

/// Computes the binary Merkle root over ordered chunk semantic commitments.
pub fn compute_root_semantic_commitment(chunk_commitments: &[[u8; 32]]) -> [u8; 32] {
    if chunk_commitments.is_empty() {
        return Sha256::digest(MERKLE_EMPTY_DOMAIN).into();
    }
    let mut level: Vec<[u8; 32]> = chunk_commitments
        .iter()
        .map(|commitment| {
            let mut hasher = Sha256::new();
            hasher.update([MERKLE_LEAF_TAG]);
            hasher.update(commitment);
            hasher.finalize().into()
        })
        .collect();
    while level.len() > 1 {
        level = level
            .chunks(2)
            .map(|pair| match pair {
                [left, right] => {
                    let mut hasher = Sha256::new();
                    hasher.update([MERKLE_NODE_TAG]);
                    hasher.update(left);
                    hasher.update(right);
                    hasher.finalize().into()
                }
                [single] => *single,
                _ => unreachable!("chunks(2) yields 1 or 2"),
            })
            .collect();
    }
    level[0]
}

fn hash_canonical_row_header(
    hasher: &mut Sha256,
    partition_id: u32,
    domain: CanonicalKeyDomain,
    kind_id: u32,
    logical_key: &[u8],
    payload: &[u8],
    history_digest: &[u8; 32],
) {
    hasher.update(partition_id.to_le_bytes());
    hasher.update([domain.tag()]);
    hasher.update(kind_id.to_le_bytes());
    hasher.update((logical_key.len() as u32).to_le_bytes());
    hasher.update(logical_key);
    hasher.update((payload.len() as u32).to_le_bytes());
    hasher.update(payload);
    hasher.update(history_digest);
}

/// Computes the consolidation-invariant semantic commitment for canonical rows.
pub fn compute_chunk_semantic_commitment(rows: &[HistoricalBasisRow]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(SEMANTIC_CHUNK_DOMAIN);
    hasher.update((rows.len() as u64).to_le_bytes());
    for row in rows {
        let hist_digest = compute_row_history_digest(&row.history_versions);
        hash_canonical_row_header(
            &mut hasher,
            row.partition_id.0,
            row.domain,
            row.kind_id.0,
            &row.logical_key,
            &row.payload,
            &hist_digest,
        );
    }
    hasher.finalize().into()
}

pub(crate) fn compute_chunk_semantic_commitment_from_physical(
    sorted_rows: &[&PhysicalChunkRow],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(SEMANTIC_CHUNK_DOMAIN);
    hasher.update((sorted_rows.len() as u64).to_le_bytes());
    for row in sorted_rows {
        hash_canonical_row_header(
            &mut hasher,
            row.partition_id,
            row.domain,
            row.kind_id,
            &row.logical_key,
            &row.payload,
            &row.history_versions_sha256,
        );
    }
    hasher.finalize().into()
}

pub(crate) fn compute_chunk_semantic_commitment_from_borrowed(
    sorted_rows: &[BorrowedChunkRow<'_>],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(SEMANTIC_CHUNK_DOMAIN);
    hasher.update((sorted_rows.len() as u64).to_le_bytes());
    for row in sorted_rows {
        hash_canonical_row_header(
            &mut hasher,
            row.partition_id,
            row.domain,
            row.kind_id,
            row.logical_key,
            row.payload,
            &row.history_versions_sha256,
        );
    }
    hasher.finalize().into()
}

/// Seals canonical rows into non-overlapping fixed chunks with per-slot continuation
/// pages and consolidation-invariant semantic commitments.
pub fn seal_basis_chunks(
    mut rows: Vec<HistoricalBasisRow>,
    geometry: HistoricalChunkGeometry,
) -> Result<SealedBasisChunks, HistoricalReadDenial> {
    let rows_per_chunk = geometry.rows_per_chunk.max(1);
    let page_versions = geometry.continuation_page_versions.max(1);
    rows.sort_by(|a, b| {
        (a.partition_id.0, a.domain, &a.logical_key).cmp(&(
            b.partition_id.0,
            b.domain,
            &b.logical_key,
        ))
    });
    for w in rows.windows(2) {
        if (w[0].partition_id, w[0].domain) == (w[1].partition_id, w[1].domain)
            && w[0].logical_key == w[1].logical_key
        {
            return Err(corrupt());
        }
    }

    let mut members = BTreeMap::new();
    let mut chunks = Vec::new();
    let mut commitments = Vec::new();

    for slice in rows.chunks(rows_per_chunk) {
        let semantic_commitment = compute_chunk_semantic_commitment(slice);
        let min_key = slice
            .first()
            .and_then(HistoricalBasisRow::canonical_cursor_key)
            .ok_or_else(corrupt)?;
        let max_key = slice
            .last()
            .and_then(HistoricalBasisRow::canonical_cursor_key)
            .ok_or_else(corrupt)?;

        let mut physical_rows = Vec::with_capacity(slice.len());
        let mut continuation_members = Vec::new();
        for row in slice {
            let total_history_bytes: u64 = row
                .history_versions
                .iter()
                .map(|v| v.len() as u64)
                .fold(0u64, u64::saturating_add);
            let history_versions_sha256 = compute_row_history_digest(&row.history_versions);
            let mut cont_ids = Vec::new();
            let inline_history = if row.history_versions.len() > geometry.inline_history_limit {
                let spill_count = row.history_versions.len() - geometry.inline_history_limit;
                let (spilled, inline) = row.history_versions.split_at(spill_count);
                for (seq, page_slice) in spilled.chunks(page_versions).enumerate() {
                    let page = ContinuationPageImage {
                        partition_id: row.partition_id.0,
                        domain: row.domain,
                        logical_key: row.logical_key.clone(),
                        sequence: seq as u32,
                        versions: page_slice.to_vec(),
                    };
                    let bytes = encode_continuation_page(&page);
                    let id = member_id(&bytes);
                    members.insert(id, bytes);
                    cont_ids.push(id);
                    continuation_members.push(id);
                }
                inline.to_vec()
            } else {
                row.history_versions.clone()
            };
            physical_rows.push(PhysicalChunkRow {
                partition_id: row.partition_id.0,
                domain: row.domain,
                kind_id: row.kind_id.0,
                total_history_bytes,
                history_versions_sha256,
                logical_key: row.logical_key.clone(),
                payload: row.payload.clone(),
                inline_history,
                continuation_pages: cont_ids,
            });
        }

        let layers = if geometry.layered_initial && physical_rows.len() >= 2 {
            let mid = physical_rows.len() / 2;
            let right = physical_rows.split_off(mid);
            vec![physical_rows, right]
        } else if geometry.layered_initial {
            vec![physical_rows, Vec::new()]
        } else {
            vec![physical_rows]
        };

        let chunk_bytes = encode_chunk_image(&PhysicalChunkImage { layers });
        let chunk_member = member_id(&chunk_bytes);
        members.insert(chunk_member, chunk_bytes);
        commitments.push(semantic_commitment);
        let continuation_count = continuation_members.len() as u32;
        chunks.push(SealedChunkDescriptor {
            semantic_commitment,
            min_key,
            max_key,
            chunk_member,
            continuation_members,
            chunk_locator: None,
            continuation_locators: Vec::new(),
            continuation_manifest_member: None,
            continuation_manifest_locator: None,
            continuation_count,
        });
    }

    let root_semantic_commitment = compute_root_semantic_commitment(&commitments);
    Ok(SealedBasisChunks {
        root_semantic_commitment,
        chunks,
        members,
    })
}

pub fn member_id(bytes: &[u8]) -> ChunkMemberId {
    ChunkMemberId {
        sha256: Sha256::digest(bytes).into(),
        length: bytes.len() as u64,
    }
}

pub fn verify_member_digest(
    expected: ChunkMemberId,
    bytes: &[u8],
) -> Result<(), HistoricalReadDenial> {
    if bytes.len() as u64 != expected.length
        || <[u8; 32]>::from(Sha256::digest(bytes)) != expected.sha256
    {
        return Err(corrupt());
    }
    Ok(())
}

pub(super) fn corrupt() -> HistoricalReadDenial {
    HistoricalReadDenial::CorruptStorage {
        subject: RelationalStorageSubject::Region(PartitionId(0)),
    }
}
