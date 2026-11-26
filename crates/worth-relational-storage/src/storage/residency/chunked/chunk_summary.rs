//! Producer-derived kind and adjacency summaries of one chunk.
//!
//! A summary is never supplied by a caller as fact. The sealer derives it from
//! the canonical projection while folding that projection into the semantic
//! commitment, and compares any producer claim against the derivation before
//! hashing. A negative summary therefore proves absence: a summary-pruned scan
//! agrees with the canonical full scan.
use std::collections::BTreeSet;

use sha2::{Digest, Sha256};

use super::slot_projection::{AdjacencyEdge, SlotVersion};
use crate::identity::data::KindId;

/// Sorted, duplicate-free record kinds and incident relation kinds.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChunkKindAdjacencySummary {
    record_kinds: Vec<KindId>,
    adjacency_kinds: Vec<KindId>,
}

impl ChunkKindAdjacencySummary {
    pub fn record_kinds(&self) -> &[KindId] {
        &self.record_kinds
    }

    pub fn adjacency_kinds(&self) -> &[KindId] {
        &self.adjacency_kinds
    }

    /// `false` proves no covered record of the chunk has `kind`.
    pub fn may_contain_kind(&self, kind: KindId) -> bool {
        self.record_kinds.binary_search(&kind).is_ok()
    }

    /// `false` proves no covered vertex has an incident `relation_kind` edge.
    pub fn may_contain_adjacency(&self, relation_kind: KindId) -> bool {
        self.adjacency_kinds.binary_search(&relation_kind).is_ok()
    }

    pub(super) fn fold_into(&self, hasher: &mut Sha256) {
        for kinds in [&self.record_kinds, &self.adjacency_kinds] {
            hasher.update((kinds.len() as u64).to_be_bytes());
            for kind in kinds {
                hasher.update(kind.as_u32().to_be_bytes());
            }
        }
    }
}

/// Accumulates a summary from canonical versions and edges.
#[derive(Debug, Default)]
pub struct SummaryDerivation {
    record_kinds: BTreeSet<KindId>,
    adjacency_kinds: BTreeSet<KindId>,
}

impl SummaryDerivation {
    pub fn observe_version(&mut self, version: &SlotVersion) {
        self.record_kinds.insert(version.kind);
    }

    pub fn observe_edge(&mut self, edge: &AdjacencyEdge) {
        self.adjacency_kinds.insert(edge.relation_kind);
    }

    pub fn finish(self) -> ChunkKindAdjacencySummary {
        ChunkKindAdjacencySummary {
            record_kinds: self.record_kinds.into_iter().collect(),
            adjacency_kinds: self.adjacency_kinds.into_iter().collect(),
        }
    }
}

/// A producer's claimed summary pair, checked against the derivation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimedChunkSummaries {
    pub head: ChunkKindAdjacencySummary,
    pub history: ChunkKindAdjacencySummary,
}

impl ClaimedChunkSummaries {
    /// Builds a claim from arbitrary kind lists, as a producer might.
    pub fn from_kinds(
        head_records: impl IntoIterator<Item = KindId>,
        head_adjacency: impl IntoIterator<Item = KindId>,
        history_records: impl IntoIterator<Item = KindId>,
    ) -> Self {
        let collect = |kinds: &mut dyn Iterator<Item = KindId>| {
            kinds
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        };
        Self {
            head: ChunkKindAdjacencySummary {
                record_kinds: collect(&mut head_records.into_iter()),
                adjacency_kinds: collect(&mut head_adjacency.into_iter()),
            },
            history: ChunkKindAdjacencySummary {
                record_kinds: collect(&mut history_records.into_iter()),
                adjacency_kinds: Vec::new(),
            },
        }
    }
}
