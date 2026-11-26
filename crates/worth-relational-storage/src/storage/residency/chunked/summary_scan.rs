//! Kind scans and adjacency traversals over an admitted chunk directory.
//!
//! A chunk is skipped only when its producer-derived summary proves absence,
//! and only an admitted [`RegionChunkDirectory`] (complete, non-overlapping
//! coverage) is accepted, so a pruned scan returns exactly what the
//! canonical full scan of every chunk returns.
use super::canonical_read::{read_chunk_canonical, ChunkReadError, ChunkResidency, SlotSelection};
use super::chunk_directory::{BoundChunk, RegionChunkDirectory};
use super::member_io::ChunkMemberSource;
use super::slot_projection::{AdjacencyEdge, CanonicalChunkVisitor, SlotVersion};
use super::slot_range::SlotDomain;
use crate::identity::data::{KindId, RelationId};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

/// What a scan selects and whether summaries may skip chunks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkScan {
    /// Slots whose head version has `kind`.
    HeadKind(KindId),
    /// Slots with any history version of `kind`.
    HistoryKind(KindId),
    /// Incident relations of `relation_kind`, by vertex slot.
    Adjacency(KindId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanPruning {
    /// Read only chunks whose summary may contain the selection.
    Summaries,
    /// Read every chunk: the canonical full scan.
    Canonical,
}

/// One scan hit: a slot, and for adjacency scans the incident relation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScanHit {
    pub slot: u64,
    pub relation: Option<RelationId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanOutcome {
    pub hits: Vec<ScanHit>,
    pub chunks_read: usize,
}

pub fn scan_chunks(
    directory: &RegionChunkDirectory,
    residency: &ChunkResidency,
    source: &dyn ChunkMemberSource,
    domain: SlotDomain,
    scan: ChunkScan,
    pruning: ScanPruning,
) -> Result<ScanOutcome, Cause> {
    let mut visitor = ScanVisitor {
        scan,
        slot: 0,
        head_hit: false,
        history_hit: false,
        hits: Vec::new(),
    };
    let mut chunks_read = 0;
    for chunk in directory.domain_chunks(domain) {
        if pruning == ScanPruning::Summaries && !may_contain(chunk, scan) {
            continue;
        }
        chunks_read += 1;
        read_chunk_canonical(
            residency,
            source,
            domain,
            chunk.binding(),
            SlotSelection::All,
            &mut visitor,
        )
        .map_err(|error| match error {
            ChunkReadError::Member(cause) => cause,
            ChunkReadError::Visitor(never) => match never {},
        })?;
    }
    Ok(ScanOutcome {
        hits: visitor.hits,
        chunks_read,
    })
}

fn may_contain(chunk: &BoundChunk, scan: ChunkScan) -> bool {
    let semantic = chunk.semantic();
    match scan {
        ChunkScan::HeadKind(kind) => semantic.head_summary().may_contain_kind(kind),
        ChunkScan::HistoryKind(kind) => semantic.history_summary().may_contain_kind(kind),
        ChunkScan::Adjacency(kind) => semantic.head_summary().may_contain_adjacency(kind),
    }
}

struct ScanVisitor {
    scan: ChunkScan,
    slot: u64,
    head_hit: bool,
    history_hit: bool,
    hits: Vec<ScanHit>,
}

impl CanonicalChunkVisitor for ScanVisitor {
    type Error = std::convert::Infallible;

    fn slot_begin(&mut self, slot: u64, head: Option<&SlotVersion>) -> Result<(), Self::Error> {
        self.slot = slot;
        self.head_hit = matches!((self.scan, head), (ChunkScan::HeadKind(kind), Some(head)) if head.kind == kind);
        self.history_hit = false;
        Ok(())
    }

    fn history_version(&mut self, version: &SlotVersion) -> Result<(), Self::Error> {
        if let ChunkScan::HistoryKind(kind) = self.scan {
            self.history_hit |= version.kind == kind;
        }
        Ok(())
    }

    fn adjacency_edge(&mut self, edge: &AdjacencyEdge) -> Result<(), Self::Error> {
        if self.scan == ChunkScan::Adjacency(edge.relation_kind) {
            self.hits.push(ScanHit {
                slot: self.slot,
                relation: Some(edge.relation),
            });
        }
        Ok(())
    }

    fn slot_end(&mut self) -> Result<(), Self::Error> {
        if self.head_hit || self.history_hit {
            self.hits.push(ScanHit {
                slot: self.slot,
                relation: None,
            });
        }
        Ok(())
    }
}
