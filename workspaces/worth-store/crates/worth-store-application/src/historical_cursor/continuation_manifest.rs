//! Bounded `<= 3 KiB` per-chunk continuation reference tree (`WHCM1`).
//!
//! Stores `(ChunkMemberId, [u8; 64])` continuation locators out-of-line from
//! chunk leaf descriptors so key-only and head-only cursors NEVER load
//! continuation references, while full-history cursors fault only O(log N)
//! bounded `<= 3 KiB` reference pages on demand under `HistoricalWorkingSetPool`.

use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::storage::RelationalStorageSubject;
use worth_runtime_world::facade::HistoricalReadDenial;

use super::chunk_format::ChunkMemberId;

const CONT_MANIFEST_MAGIC: &[u8; 5] = b"WHCM1";
const CONT_MANIFEST_SEAL_DOMAIN: &[u8] = b"worth.historical.cont-manifest.v1";
const CONT_TAG_LEAF: u8 = 0;
const CONT_TAG_INDEX: u8 = 1;
const MAX_CONT_PAGE_BYTES: usize = 3072;
const MAX_CONT_DEPTH: usize = 16;

const LEAF_ENTRY_BYTES: usize = 32 + 8 + 64;
const INDEX_ENTRY_BYTES: usize = 32 + 8 + 32 + 8 + 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ContRefEntry {
    pub(super) member: ChunkMemberId,
    pub(super) locator: [u8; 64],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ContIndexChild {
    pub(super) max_member: ChunkMemberId,
    pub(super) member: ChunkMemberId,
    pub(super) locator: [u8; 64],
}

enum ContManifestNode {
    Leaf(Vec<ContRefEntry>),
    Index(Vec<ContIndexChild>),
}

/// Greedily partitions sorted continuation refs into strictly bounded `<= 3 KiB` leaf pages.
pub(super) fn partition_continuation_leaves(
    entries: &[ContRefEntry],
) -> Result<Vec<(ChunkMemberId, Vec<u8>)>, HistoricalReadDenial> {
    partition_bounded(
        entries,
        |entry| entry.member,
        |slice| {
            encode_cont_node(CONT_TAG_LEAF, slice, |body, entry| {
                body.extend_from_slice(&entry.member.sha256);
                body.extend_from_slice(&entry.member.length.to_le_bytes());
                body.extend_from_slice(&entry.locator);
            })
        },
    )
}

/// Greedily partitions ordered continuation index children into `<= 3 KiB` index pages.
pub(super) fn partition_continuation_index(
    children: &[ContIndexChild],
) -> Result<Vec<(ChunkMemberId, Vec<u8>)>, HistoricalReadDenial> {
    partition_bounded(
        children,
        |child| child.max_member,
        |slice| {
            encode_cont_node(CONT_TAG_INDEX, slice, |body, child| {
                body.extend_from_slice(&child.max_member.sha256);
                body.extend_from_slice(&child.max_member.length.to_le_bytes());
                body.extend_from_slice(&child.member.sha256);
                body.extend_from_slice(&child.member.length.to_le_bytes());
                body.extend_from_slice(&child.locator);
            })
        },
    )
}

fn partition_bounded<T>(
    items: &[T],
    max_id_of: impl Fn(&T) -> ChunkMemberId,
    encode: impl Fn(&[T]) -> Vec<u8>,
) -> Result<Vec<(ChunkMemberId, Vec<u8>)>, HistoricalReadDenial> {
    let mut pages = Vec::new();
    let mut start = 0usize;
    while start < items.len() {
        let mut end = start + 1;
        let mut best = encode(&items[start..end]);
        if best.len() > MAX_CONT_PAGE_BYTES {
            return Err(corrupt());
        }
        while end < items.len() {
            let candidate = encode(&items[start..=end]);
            if candidate.len() > MAX_CONT_PAGE_BYTES {
                break;
            }
            best = candidate;
            end += 1;
        }
        pages.push((max_id_of(&items[end - 1]), best));
        start = end;
    }
    Ok(pages)
}

fn encode_cont_node<T>(tag: u8, items: &[T], write_item: impl Fn(&mut Vec<u8>, &T)) -> Vec<u8> {
    let mut body = Vec::with_capacity(CONT_MANIFEST_MAGIC.len() + 5 + items.len() * 144);
    body.extend_from_slice(CONT_MANIFEST_MAGIC);
    body.push(tag);
    body.extend_from_slice(&(items.len() as u32).to_le_bytes());
    for item in items {
        write_item(&mut body, item);
    }
    body.extend_from_slice(&seal(&body));
    body
}

fn decode_cont_node(bytes: &[u8]) -> Result<ContManifestNode, HistoricalReadDenial> {
    if bytes.len() < CONT_MANIFEST_MAGIC.len() + 1 + 4 + 32 || bytes.len() > MAX_CONT_PAGE_BYTES {
        return Err(corrupt());
    }
    let (body, tag) = bytes.split_at(bytes.len() - 32);
    if tag != seal(body) || &body[..5] != CONT_MANIFEST_MAGIC {
        return Err(corrupt());
    }
    let node_tag = body[5];
    let count = u32::from_le_bytes(body[6..10].try_into().expect("4")) as usize;
    let payload = &body[10..];
    match node_tag {
        CONT_TAG_LEAF => {
            if payload.len() != count.saturating_mul(LEAF_ENTRY_BYTES) {
                return Err(corrupt());
            }
            let mut entries = Vec::with_capacity(count);
            for chunk in payload.chunks_exact(LEAF_ENTRY_BYTES) {
                entries.push(ContRefEntry {
                    member: ChunkMemberId {
                        sha256: chunk[0..32].try_into().expect("32"),
                        length: u64::from_le_bytes(chunk[32..40].try_into().expect("8")),
                    },
                    locator: chunk[40..104].try_into().expect("64"),
                });
            }
            Ok(ContManifestNode::Leaf(entries))
        }
        CONT_TAG_INDEX => {
            if payload.len() != count.saturating_mul(INDEX_ENTRY_BYTES) {
                return Err(corrupt());
            }
            let mut children = Vec::with_capacity(count);
            for chunk in payload.chunks_exact(INDEX_ENTRY_BYTES) {
                children.push(ContIndexChild {
                    max_member: ChunkMemberId {
                        sha256: chunk[0..32].try_into().expect("32"),
                        length: u64::from_le_bytes(chunk[32..40].try_into().expect("8")),
                    },
                    member: ChunkMemberId {
                        sha256: chunk[40..72].try_into().expect("32"),
                        length: u64::from_le_bytes(chunk[72..80].try_into().expect("8")),
                    },
                    locator: chunk[80..144].try_into().expect("64"),
                });
            }
            Ok(ContManifestNode::Index(children))
        }
        _ => Err(corrupt()),
    }
}

/// Binary-searches a chunk's paged continuation reference tree for `target`,
/// faulting only O(log N) bounded `<= 3 KiB` pages on demand.
pub(super) fn lookup_continuation_locator<F>(
    root_id: ChunkMemberId,
    root_locator: Option<&[u8; 64]>,
    target: ChunkMemberId,
    mut load_node: F,
) -> Result<Option<[u8; 64]>, HistoricalReadDenial>
where
    F: FnMut(ChunkMemberId, Option<&[u8; 64]>) -> Result<Arc<Vec<u8>>, HistoricalReadDenial>,
{
    let mut cur_id = root_id;
    let mut cur_loc = root_locator.copied();
    for _ in 0..MAX_CONT_DEPTH {
        let bytes = load_node(cur_id, cur_loc.as_ref())?;
        let node = decode_cont_node(&bytes)?;
        drop(bytes);
        match node {
            ContManifestNode::Leaf(entries) => {
                return Ok(entries
                    .binary_search_by_key(&target, |entry| entry.member)
                    .ok()
                    .map(|idx| entries[idx].locator));
            }
            ContManifestNode::Index(children) => {
                let idx = children.partition_point(|child| child.max_member < target);
                let Some(child) = children.get(idx) else {
                    return Ok(None);
                };
                cur_id = child.member;
                cur_loc = Some(child.locator);
            }
        }
    }
    Err(corrupt())
}

/// Collects every continuation reference of one chunk (used during consolidation).
pub(super) fn collect_all_continuation_refs<F>(
    root_id: ChunkMemberId,
    root_locator: Option<&[u8; 64]>,
    mut load_node: F,
) -> Result<Vec<ContRefEntry>, HistoricalReadDenial>
where
    F: FnMut(ChunkMemberId, Option<&[u8; 64]>) -> Result<Arc<Vec<u8>>, HistoricalReadDenial>,
{
    let mut out = Vec::new();
    let mut stack = vec![(root_id, root_locator.copied(), 0usize)];
    while let Some((id, loc, depth)) = stack.pop() {
        if depth >= MAX_CONT_DEPTH {
            return Err(corrupt());
        }
        let bytes = load_node(id, loc.as_ref())?;
        let node = decode_cont_node(&bytes)?;
        drop(bytes);
        match node {
            ContManifestNode::Leaf(entries) => out.extend(entries),
            ContManifestNode::Index(children) => {
                for child in children.into_iter().rev() {
                    stack.push((child.member, Some(child.locator), depth + 1));
                }
            }
        }
    }
    Ok(out)
}

fn seal(body: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(CONT_MANIFEST_SEAL_DOMAIN);
    h.update(body);
    h.finalize().into()
}

fn corrupt() -> HistoricalReadDenial {
    HistoricalReadDenial::CorruptStorage {
        subject: RelationalStorageSubject::Region(PartitionId(0)),
    }
}
