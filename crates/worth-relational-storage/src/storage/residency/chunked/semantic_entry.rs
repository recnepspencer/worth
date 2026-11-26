//! Consolidation-invariant semantic chunk commitments and their Merkle root.
//!
//! The commitment depends only on the canonical visit order of
//! [`CanonicalChunkVisitor`]: no physical locator, layer boundary or
//! continuation page boundary is ever hashed, so flattening layered history
//! or re-paging a spilled slot leaves it bit-identical.
use sha2::{Digest, Sha256};

use super::chunk_summary::{ChunkKindAdjacencySummary, SummaryDerivation};
use super::slot_projection::{AdjacencyEdge, CanonicalChunkVisitor, SlotVersion};
use super::slot_range::{SlotDomain, SlotRange};

const CHUNK_DOMAIN: &[u8] = b"worth.relational.chunk.semantic.v1";
const SLOT_DOMAIN: &[u8] = b"worth.relational.chunk.slot.v1";
const MERKLE_LEAF: u8 = 0;
const MERKLE_NODE: u8 = 1;
const MERKLE_EMPTY: &[u8] = b"worth.relational.chunk.merkle.empty.v1";

/// One chunk's canonical meaning, folded into the selected root's Merkle
/// commitment. Only the chunk sealer and the canonical verifier mint it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkSemanticEntry {
    domain: SlotDomain,
    slot_range: SlotRange,
    semantic_chunk_commitment: [u8; 32],
    head_summary: ChunkKindAdjacencySummary,
    history_summary: ChunkKindAdjacencySummary,
}

impl ChunkSemanticEntry {
    pub const fn domain(&self) -> SlotDomain {
        self.domain
    }

    pub const fn slot_range(&self) -> SlotRange {
        self.slot_range
    }

    pub const fn semantic_chunk_commitment(&self) -> [u8; 32] {
        self.semantic_chunk_commitment
    }

    pub const fn head_summary(&self) -> &ChunkKindAdjacencySummary {
        &self.head_summary
    }

    pub const fn history_summary(&self) -> &ChunkKindAdjacencySummary {
        &self.history_summary
    }

    fn leaf_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update([MERKLE_LEAF, self.domain.tag()]);
        hasher.update(self.slot_range.commit_bytes());
        hasher.update(self.semantic_chunk_commitment);
        self.head_summary.fold_into(&mut hasher);
        self.history_summary.fold_into(&mut hasher);
        hasher.finalize().into()
    }
}

/// Binary Merkle root over ordered chunk entries; an odd node is promoted.
pub fn root_semantic_merkle(entries: &[ChunkSemanticEntry]) -> [u8; 32] {
    let mut level = entries
        .iter()
        .map(ChunkSemanticEntry::leaf_hash)
        .collect::<Vec<_>>();
    if level.is_empty() {
        return Sha256::digest(MERKLE_EMPTY).into();
    }
    while level.len() > 1 {
        level = level
            .chunks(2)
            .map(|pair| match pair {
                [left, right] => {
                    let mut hasher = Sha256::new();
                    hasher.update([MERKLE_NODE]);
                    hasher.update(left);
                    hasher.update(right);
                    hasher.finalize().into()
                }
                [single] => *single,
                _ => unreachable!("chunks(2) yields one or two nodes"),
            })
            .collect();
    }
    level[0]
}

/// A chunk's canonical content digest and its derived summaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedChunkMeaning {
    domain: SlotDomain,
    slot_range: SlotRange,
    content_digest: [u8; 32],
    head_summary: ChunkKindAdjacencySummary,
    history_summary: ChunkKindAdjacencySummary,
}

impl DerivedChunkMeaning {
    pub fn head_summary(&self) -> &ChunkKindAdjacencySummary {
        &self.head_summary
    }

    pub fn history_summary(&self) -> &ChunkKindAdjacencySummary {
        &self.history_summary
    }

    /// Folds the content digest and the derived summaries into the entry.
    pub fn commit(self) -> ChunkSemanticEntry {
        let mut hasher = Sha256::new();
        hasher.update(CHUNK_DOMAIN);
        hasher.update([self.domain.tag()]);
        hasher.update(self.slot_range.commit_bytes());
        hasher.update(self.content_digest);
        self.head_summary.fold_into(&mut hasher);
        self.history_summary.fold_into(&mut hasher);
        ChunkSemanticEntry {
            domain: self.domain,
            slot_range: self.slot_range,
            semantic_chunk_commitment: hasher.finalize().into(),
            head_summary: self.head_summary,
            history_summary: self.history_summary,
        }
    }
}

/// Derives a chunk's semantic entry from its canonical visit. Summaries are
/// derived here, from the same stream that is hashed.
pub struct ChunkCommitmentHasher {
    domain: SlotDomain,
    slot_range: SlotRange,
    content: Sha256,
    slot: Option<SlotFold>,
    head: SummaryDerivation,
    history: SummaryDerivation,
}

struct SlotFold {
    hasher: Sha256,
    history_versions: u64,
    adjacency_edges: u64,
}

impl ChunkCommitmentHasher {
    pub fn new(domain: SlotDomain, slot_range: SlotRange) -> Self {
        Self {
            domain,
            slot_range,
            content: Sha256::new(),
            slot: None,
            head: SummaryDerivation::default(),
            history: SummaryDerivation::default(),
        }
    }

    /// Ends the visit with the derived summaries and content digest. The
    /// commitment is computed only by [`DerivedChunkMeaning::commit`], after
    /// any producer claim has been checked against these derivations.
    pub fn finish(self) -> DerivedChunkMeaning {
        debug_assert!(self.slot.is_none(), "every visited slot ends before finish");
        DerivedChunkMeaning {
            domain: self.domain,
            slot_range: self.slot_range,
            content_digest: self.content.finalize().into(),
            head_summary: self.head.finish(),
            history_summary: self.history.finish(),
        }
    }

    fn open_slot(&mut self) -> &mut SlotFold {
        self.slot
            .as_mut()
            .expect("slot_begin precedes slot entries")
    }
}

fn fold_version(hasher: &mut Sha256, version: &SlotVersion) {
    hasher.update(version.kind.as_u32().to_be_bytes());
    hasher.update((version.payload.len() as u64).to_be_bytes());
    hasher.update(&version.payload);
}

impl CanonicalChunkVisitor for ChunkCommitmentHasher {
    type Error = std::convert::Infallible;

    fn slot_begin(&mut self, slot: u64, head: Option<&SlotVersion>) -> Result<(), Self::Error> {
        let mut hasher = Sha256::new();
        hasher.update(SLOT_DOMAIN);
        hasher.update(slot.to_be_bytes());
        match head {
            Some(version) => {
                hasher.update([1]);
                fold_version(&mut hasher, version);
                self.head.observe_version(version);
            }
            None => hasher.update([0]),
        }
        self.slot = Some(SlotFold {
            hasher,
            history_versions: 0,
            adjacency_edges: 0,
        });
        Ok(())
    }

    fn history_version(&mut self, version: &SlotVersion) -> Result<(), Self::Error> {
        self.history.observe_version(version);
        let slot = self.open_slot();
        slot.hasher.update([2]);
        fold_version(&mut slot.hasher, version);
        slot.history_versions += 1;
        Ok(())
    }

    fn adjacency_edge(&mut self, edge: &AdjacencyEdge) -> Result<(), Self::Error> {
        self.head.observe_edge(edge);
        let slot = self.open_slot();
        slot.hasher.update([3]);
        slot.hasher
            .update(edge.relation_kind.as_u32().to_be_bytes());
        slot.hasher
            .update(edge.relation.partition_value().to_be_bytes());
        slot.hasher
            .update(edge.relation.local_slot_value().to_be_bytes());
        slot.hasher
            .update(edge.relation.generation_value().to_be_bytes());
        slot.adjacency_edges += 1;
        Ok(())
    }

    fn slot_end(&mut self) -> Result<(), Self::Error> {
        let mut slot = self.slot.take().expect("slot_begin precedes slot_end");
        slot.hasher.update(slot.history_versions.to_be_bytes());
        slot.hasher.update(slot.adjacency_edges.to_be_bytes());
        self.content.update(slot.hasher.finalize());
        Ok(())
    }
}
