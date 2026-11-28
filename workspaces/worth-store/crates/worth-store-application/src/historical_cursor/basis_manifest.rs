//! Paged per-basis chunk descriptor manifest tree (`WHBM4`) stored as bounded
//! content-addressed anchored record members (Store Milestone 4, Phase M4.4).
//!
//! Guarantees:
//! 1. Restoring a Directory Home on cold open allocates ZERO chunk or continuation
//!    descriptors for unrelated retained bases.
//! 2. Continuation references are paged separately (`WHCM1`), so chunk leaf
//!    descriptors stay compact and key-only / head-only cursors NEVER load
//!    continuation references.
//! 3. Internal index pages store bounded separator keys (`<= 256` bytes) PLUS a
//!    fixed-size `rightmost_leaf` pointer, guaranteeing fan-out `>= 6` per
//!    `3 KiB` index page and strictly `O(log N)` bounded page reads per seek
//!    even when all keys share identical >256-byte prefixes!
//! 4. Oversized pages and non-shrinking index levels are explicitly rejected.

use std::sync::Arc;

use worth_runtime_world::facade::{CanonicalCursorKey, CanonicalKeyDomain, HistoricalReadDenial};

use super::basis_manifest_codec::{
    corrupt, encode_chunk_descriptor, manifest_seal, read_child_ref, read_chunk_descriptor,
    write_child_ref, Reader,
};
use super::chunk_format::{ChunkMemberId, SealedChunkDescriptor};

const MANIFEST_MAGIC: &[u8; 5] = b"WHBM4";
const NODE_TAG_LEAF: u8 = 0;
const NODE_TAG_INDEX: u8 = 1;
pub(super) const TARGET_MANIFEST_PAGE_BYTES: usize = 3072;
const MAX_LEAF_MANIFEST_PAGE_BYTES: usize = 4608;
pub(super) const MAX_INDEX_SEPARATOR_BYTES: usize = 256;
const MAX_MANIFEST_DEPTH: usize = 16;

const MIN_CHUNK_BYTES: usize = 32 + 1 + 32 + 8 + 1 + 1 + 1 + 4;
const MIN_CHILD_BYTES: usize = 1 + 32 + 8 + 64 + 32 + 8 + 64;

/// Bounded upper-bound routing key stored in internal manifest index pages.
///
/// Always `<= 256` prefix bytes (`truncated = true` when the full `max_key` was
/// longer), preserving sound upper-bound and lower-bound pruning while keeping
/// internal index pages compact (`<= 3 KiB`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RouteMaxKey {
    pub(super) is_start: bool,
    pub(super) partition_id: u32,
    pub(super) domain: CanonicalKeyDomain,
    pub(super) prefix: Vec<u8>,
    pub(super) truncated: bool,
}

impl RouteMaxKey {
    pub(super) fn from_cursor_key(key: &CanonicalCursorKey) -> Self {
        match key {
            CanonicalCursorKey::Start => Self {
                is_start: true,
                partition_id: 0,
                domain: CanonicalKeyDomain::Entity,
                prefix: Vec::new(),
                truncated: false,
            },
            CanonicalCursorKey::After {
                partition_id,
                domain,
                logical_key,
            } => {
                let truncated = logical_key.len() > MAX_INDEX_SEPARATOR_BYTES;
                let prefix_len = logical_key.len().min(MAX_INDEX_SEPARATOR_BYTES);
                Self {
                    is_start: false,
                    partition_id: *partition_id,
                    domain: *domain,
                    prefix: logical_key[..prefix_len].to_vec(),
                    truncated,
                }
            }
        }
    }

    /// Sound upper-bound test: returns `true` ONLY when every key `<= full_max`
    /// in this subtree is strictly `<= after`, so skipping this child can never
    /// miss a key `> after`.
    pub(super) fn is_guaranteed_le(&self, after: &CanonicalCursorKey) -> bool {
        let CanonicalCursorKey::After {
            partition_id: after_pid,
            domain: after_dom,
            logical_key: after_key,
        } = after
        else {
            return false;
        };
        if self.is_start {
            return true;
        }
        match (self.partition_id, self.domain).cmp(&(*after_pid, *after_dom)) {
            std::cmp::Ordering::Less => true,
            std::cmp::Ordering::Greater => false,
            std::cmp::Ordering::Equal => {
                if !self.truncated {
                    self.prefix.as_slice() <= after_key.as_slice()
                } else {
                    let cmp_len = self.prefix.len().min(after_key.len());
                    self.prefix[..cmp_len] < after_key[..cmp_len]
                }
            }
        }
    }

    /// Sound lower-bound test: returns `true` ONLY when the subtree's full
    /// `max_key` is guaranteed strictly `> after`, narrowing the binary-search
    /// upper bound with zero extra page reads whenever keys differ within 256 bytes.
    pub(super) fn is_guaranteed_gt(&self, after: &CanonicalCursorKey) -> bool {
        let CanonicalCursorKey::After {
            partition_id: after_pid,
            domain: after_dom,
            logical_key: after_key,
        } = after
        else {
            return !self.is_start;
        };
        if self.is_start {
            return false;
        }
        match (self.partition_id, self.domain).cmp(&(*after_pid, *after_dom)) {
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Equal => {
                self.prefix.as_slice() > after_key.as_slice()
                    || (self.truncated && self.prefix.as_slice() == after_key.as_slice())
            }
        }
    }
}

/// Child pointer inside an internal `BasisManifestNode::Index` page, carrying
/// both the child subtree pointer and its rightmost leaf pointer for exact
/// `O(log fanout)` binary search disambiguation on shared >256-byte prefixes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BasisManifestChildRef {
    pub(super) route_max: RouteMaxKey,
    pub(super) member: ChunkMemberId,
    pub(super) locator: [u8; 64],
    pub(super) rightmost_leaf_member: ChunkMemberId,
    pub(super) rightmost_leaf_locator: [u8; 64],
}

/// Decoded bounded node in a per-basis `BasisManifest` tree.
#[derive(Debug, Clone)]
pub(super) enum BasisManifestNode {
    Leaf(Arc<[SealedChunkDescriptor]>),
    Index(Arc<[BasisManifestChildRef]>),
}

fn partition_pages<T, M>(
    items: &[T],
    target_bytes: usize,
    max_single_page_bytes: usize,
    meta_of: impl Fn(&T) -> M,
    encode: impl Fn(&[T]) -> Vec<u8>,
) -> Result<Vec<(M, Vec<u8>)>, HistoricalReadDenial> {
    let mut pages = Vec::new();
    let mut start = 0usize;
    while start < items.len() {
        let mut end = start + 1;
        let mut best_bytes = encode(&items[start..end]);
        if best_bytes.len() > max_single_page_bytes {
            return Err(corrupt());
        }
        while end < items.len() {
            let candidate = encode(&items[start..=end]);
            if candidate.len() > target_bytes {
                break;
            }
            best_bytes = candidate;
            end += 1;
        }
        pages.push((meta_of(&items[end - 1]), best_bytes));
        start = end;
    }
    Ok(pages)
}

fn encode_node<T>(tag: u8, items: &[T], write_item: impl Fn(&mut Vec<u8>, &T)) -> Vec<u8> {
    let mut body = Vec::with_capacity(MANIFEST_MAGIC.len() + 5 + items.len() * 208);
    body.extend_from_slice(MANIFEST_MAGIC);
    body.push(tag);
    body.extend_from_slice(&(items.len() as u32).to_le_bytes());
    for item in items {
        write_item(&mut body, item);
    }
    body.extend_from_slice(&manifest_seal(&body));
    body
}

/// Greedily partitions ordered chunks into bounded leaf manifest pages
/// (without inline continuation vectors).
pub(super) fn partition_manifest_leaves(
    chunks: &[SealedChunkDescriptor],
) -> Result<Vec<(RouteMaxKey, Vec<u8>)>, HistoricalReadDenial> {
    partition_pages(
        chunks,
        TARGET_MANIFEST_PAGE_BYTES,
        MAX_LEAF_MANIFEST_PAGE_BYTES,
        |c| RouteMaxKey::from_cursor_key(&c.max_key),
        |s| encode_node(NODE_TAG_LEAF, s, encode_chunk_descriptor),
    )
}

/// Metadata propagated upward by [`partition_manifest_index`]: bounded separator
/// plus the exact rightmost leaf `(member, locator)` of the partitioned subtree.
pub(super) struct ManifestIndexPageMeta {
    pub(super) route_max: RouteMaxKey,
    pub(super) rightmost_leaf_member: ChunkMemberId,
    pub(super) rightmost_leaf_locator: [u8; 64],
}

/// Greedily partitions ordered child refs into strictly `<= 3 KiB` index pages.
pub(super) fn partition_manifest_index(
    children: &[BasisManifestChildRef],
) -> Result<Vec<(ManifestIndexPageMeta, Vec<u8>)>, HistoricalReadDenial> {
    partition_pages(
        children,
        TARGET_MANIFEST_PAGE_BYTES,
        TARGET_MANIFEST_PAGE_BYTES,
        |c| ManifestIndexPageMeta {
            route_max: c.route_max.clone(),
            rightmost_leaf_member: c.rightmost_leaf_member,
            rightmost_leaf_locator: c.rightmost_leaf_locator,
        },
        |s| encode_node(NODE_TAG_INDEX, s, write_child_ref),
    )
}

/// Decodes and integrity-verifies one bounded `BasisManifestNode`.
pub(super) fn decode_basis_manifest_node(
    bytes: &[u8],
) -> Result<BasisManifestNode, HistoricalReadDenial> {
    if bytes.len() < MANIFEST_MAGIC.len() + 1 + 4 + 32 || bytes.len() > MAX_LEAF_MANIFEST_PAGE_BYTES
    {
        return Err(corrupt());
    }
    let (body, tag) = bytes.split_at(bytes.len() - 32);
    if tag != manifest_seal(body) {
        return Err(corrupt());
    }
    let mut r = Reader { bytes: body, at: 0 };
    if r.take(5)? != MANIFEST_MAGIC {
        return Err(corrupt());
    }
    let node = match r.u8()? {
        NODE_TAG_LEAF => {
            let chunk_count = r.bounded_count(MIN_CHUNK_BYTES)?;
            let mut chunks = Vec::with_capacity(chunk_count);
            for _ in 0..chunk_count {
                chunks.push(read_chunk_descriptor(&mut r)?);
            }
            BasisManifestNode::Leaf(Arc::from(chunks))
        }
        NODE_TAG_INDEX => {
            if bytes.len() > TARGET_MANIFEST_PAGE_BYTES {
                return Err(corrupt());
            }
            let child_count = r.bounded_count(MIN_CHILD_BYTES)?;
            let mut children = Vec::with_capacity(child_count);
            for _ in 0..child_count {
                children.push(read_child_ref(&mut r)?);
            }
            BasisManifestNode::Index(Arc::from(children))
        }
        _ => return Err(corrupt()),
    };
    if r.at != body.len() {
        return Err(corrupt());
    }
    Ok(node)
}

/// Walks a paged `BasisManifest` tree from `root` to collect every chunk
/// descriptor in canonical order (used for offline consolidation / inspection).
pub(super) fn collect_all_manifest_chunks<F>(
    root_id: ChunkMemberId,
    root_locator: Option<&[u8; 64]>,
    mut load_node: F,
) -> Result<Arc<[SealedChunkDescriptor]>, HistoricalReadDenial>
where
    F: FnMut(ChunkMemberId, Option<&[u8; 64]>) -> Result<Arc<Vec<u8>>, HistoricalReadDenial>,
{
    let mut out = Vec::new();
    let mut stack = vec![(root_id, root_locator.copied(), 0usize)];
    while let Some((node_id, node_loc, depth)) = stack.pop() {
        if depth >= MAX_MANIFEST_DEPTH {
            return Err(corrupt());
        }
        let bytes = load_node(node_id, node_loc.as_ref())?;
        let node = decode_basis_manifest_node(&bytes)?;
        drop(bytes);
        match node {
            BasisManifestNode::Leaf(chunks) => out.extend(chunks.iter().cloned()),
            BasisManifestNode::Index(children) => {
                for child in children.iter().rev() {
                    stack.push((child.member, Some(child.locator), depth + 1));
                }
            }
        }
    }
    Ok(Arc::from(out))
}

/// Navigates a paged `BasisManifest` tree directly to the first leaf page
/// containing a chunk strictly after `after` (returns `Ok(None)` past end).
///
/// Narrows `[lo, hi)` using bounded `RouteMaxKey` bounds first (0 extra reads
/// when keys differ within 256 bytes), then resolves any remaining shared-prefix
/// ambiguity via `O(log fanout)` binary search reading `rightmost_leaf_locator`
/// so every seek performs strictly `O(log N)` bounded page reads!
pub(super) fn load_manifest_leaf_after<F>(
    root_id: ChunkMemberId,
    root_locator: Option<&[u8; 64]>,
    after: &CanonicalCursorKey,
    mut load_node: F,
) -> Result<Option<Arc<[SealedChunkDescriptor]>>, HistoricalReadDenial>
where
    F: FnMut(ChunkMemberId, Option<&[u8; 64]>) -> Result<Arc<Vec<u8>>, HistoricalReadDenial>,
{
    let mut cur_id = root_id;
    let mut cur_loc = root_locator.copied();
    for _ in 0..MAX_MANIFEST_DEPTH {
        let bytes = load_node(cur_id, cur_loc.as_ref())?;
        let node = decode_basis_manifest_node(&bytes)?;
        drop(bytes);
        match node {
            BasisManifestNode::Leaf(chunks) => {
                if after.is_start() || chunks.last().is_some_and(|last| &last.max_key > after) {
                    return Ok(Some(chunks));
                }
                return Ok(None);
            }
            BasisManifestNode::Index(children) => {
                if children.is_empty() {
                    return Ok(None);
                }
                if after.is_start() {
                    cur_id = children[0].member;
                    cur_loc = Some(children[0].locator);
                    continue;
                }
                let lo = children
                    .iter()
                    .position(|child| !child.route_max.is_guaranteed_le(after))
                    .unwrap_or(children.len());
                if lo == children.len() {
                    return Ok(None);
                }
                let hi = children[lo..]
                    .iter()
                    .position(|child| child.route_max.is_guaranteed_gt(after))
                    .map_or(children.len(), |offset| lo + offset);
                let (mut l, mut r) = (lo, hi);
                let mut cached_leaf: Option<(usize, Arc<[SealedChunkDescriptor]>)> = None;
                while l < r {
                    let mid = l + (r - l) / 2;
                    let child = &children[mid];
                    let leaf_bytes = load_node(
                        child.rightmost_leaf_member,
                        Some(&child.rightmost_leaf_locator),
                    )?;
                    let leaf_node = decode_basis_manifest_node(&leaf_bytes)?;
                    drop(leaf_bytes);
                    let BasisManifestNode::Leaf(leaf_chunks) = leaf_node else {
                        return Err(corrupt());
                    };
                    if leaf_chunks.last().is_some_and(|last| &last.max_key > after) {
                        if child.member == child.rightmost_leaf_member {
                            cached_leaf = Some((mid, leaf_chunks));
                        }
                        r = mid;
                    } else {
                        l = mid + 1;
                    }
                }
                if l == children.len() {
                    return Ok(None);
                }
                if let Some((idx, leaf_chunks)) = cached_leaf
                    && idx == l
                {
                    return Ok(Some(leaf_chunks));
                }
                cur_id = children[l].member;
                cur_loc = Some(children[l].locator);
            }
        }
    }
    Err(corrupt())
}
