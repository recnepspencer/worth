//! Cold reads of one readmitted region leaf's chunk placement.
//!
//! The reader admits the leaf's placement (exact coverage, root and roster),
//! then faults chunk images, continuation pages and continuation-directory
//! pages through a [`ChunkResidency`] enrolled on the home's working set, from
//! the leaf's own retained cold custody. Scans skip a chunk only when its
//! producer-derived head summary proves the selection absent; skipped chunks
//! fault no member. The reader is scan-scoped: it holds a serving retain of
//! the cell's member custody only while it lives.
use std::sync::Arc;

use super::canonical_read::ChunkResidency;
use super::chunk_directory::RegionChunkDirectory;
use super::cold_member_port::ColdMemberSource;
use super::member_io::ChunkMemberSource;
use super::region_placement::{RegionChunkPlacement, REGION_CHUNK_LAYOUT};
use super::slot_range::SlotDomain;
use super::summary_scan::{scan_chunks, ChunkScan, ScanOutcome, ScanPruning};
use crate::durability::medium::NativeSourceMemberState;
use crate::identity::data::KindId;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::residency::working_set::{ResidencyPoolKind, WorkingSetAccountant};

/// Selections a recorded region placement answers. Chunks commit to current
/// heads and adjacency only, so history selections are not offered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordedChunkScan {
    /// Occupied slots of `domain` whose current head has `kind`.
    HeadKind(SlotDomain, KindId),
    /// Current outgoing relations of `relation_kind`, by entity slot.
    Adjacency(KindId),
}

pub struct RecordedRegionChunks {
    directory: RegionChunkDirectory,
    residency: ChunkResidency,
    source: ColdMemberSource,
}

impl RecordedRegionChunks {
    /// Admits `placement` and binds its reads to `custody` and the working set.
    pub fn open(
        placement: &RegionChunkPlacement,
        custody: NativeSourceMemberState,
        accountant: &Arc<WorkingSetAccountant>,
        pool: ResidencyPoolKind,
    ) -> Result<Self, Cause> {
        let directory = placement.admit().map_err(|_| Cause::IntegrityMismatch)?;
        Ok(Self {
            directory,
            residency: ChunkResidency::new(accountant, pool),
            source: ColdMemberSource::with_custody(
                custody,
                REGION_CHUNK_LAYOUT.max_chunk_image_bytes as u64,
            ),
        })
    }

    pub fn directory(&self) -> &RegionChunkDirectory {
        &self.directory
    }

    pub fn residency(&self) -> &ChunkResidency {
        &self.residency
    }

    pub fn source(&self) -> &dyn ChunkMemberSource {
        &self.source
    }

    pub fn scan(
        &self,
        scan: RecordedChunkScan,
        pruning: ScanPruning,
    ) -> Result<ScanOutcome, Cause> {
        let (domain, scan) = match scan {
            RecordedChunkScan::HeadKind(domain, kind) => (domain, ChunkScan::HeadKind(kind)),
            RecordedChunkScan::Adjacency(kind) => (SlotDomain::Entity, ChunkScan::Adjacency(kind)),
        };
        scan_chunks(
            &self.directory,
            &self.residency,
            &self.source,
            domain,
            scan,
            pruning,
        )
    }
}
