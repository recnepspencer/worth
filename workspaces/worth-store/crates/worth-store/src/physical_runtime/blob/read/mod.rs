mod selection;
mod traversal;

use std::num::NonZeroU64;

use worth_store_physical_format::{BlobRecordDenial, PersistedRecordIdentity};

use crate::physical_runtime::{
    layout::{PhysicalIndexPointKeyDenial, PhysicalLayoutDenial},
    BlobPhysicalAllocation, PhysicalReadProtectionDenial, PhysicalRecordReader,
    PhysicalScopedAllocationFailure, RecordReadError, RecordScanError, RecordStreamFailure,
    ServingPhysicalRuntime,
};

use super::{AdmittedBlobScope, PublishedBlobGeneration};

pub(in crate::physical_runtime) use selection::selected_blob_identity_exists;
pub(in crate::physical_runtime::blob) use selection::{
    selected_generation_publication, selected_session_declaration, walk_selected_reader,
};

/// A caller-admitted bound for identity issuance's selected-root walk.
/// Indexed publication resolution and reads do not perform that walk; the
/// legacy argument remains accepted by those methods for API compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobReadLimits {
    maximum_scanned_records: NonZeroU64,
}

impl BlobReadLimits {
    pub const fn new(maximum_scanned_records: NonZeroU64) -> Self {
        Self {
            maximum_scanned_records,
        }
    }

    pub const fn maximum_scanned_records(self) -> NonZeroU64 {
        self.maximum_scanned_records
    }
}

#[derive(Debug)]
pub enum BlobReadOpenFailure {
    ForeignStore,
    InvalidRequestedIdentity,
    ScopeMismatch,
    RootProtection(PhysicalReadProtectionDenial),
    InvalidPointKey(PhysicalIndexPointKeyDenial),
    Layout(PhysicalLayoutDenial),
    Allocation(PhysicalScopedAllocationFailure),
    ScratchUnavailable,
    Scan(RecordScanError),
    ScanBoundExhausted,
    PublicationNotFound,
    ConflictingPublication,
    PublicationDamaged(BlobRecordDenial),
    RangeOutOfBounds,
    Read(BlobReadFailure),
}

#[derive(Debug)]
pub enum BlobReadFailure {
    RecordRead(RecordReadError),
    RecordStream(RecordStreamFailure),
    Format(BlobRecordDenial),
    ScratchUnavailable,
    ReuseAuthority(super::BlobDedupeFailure),
    TreeDamaged,
    ReuseClaimNotRelocatable,
    ChunkCorruption {
        ordinal: u64,
        record: PersistedRecordIdentity,
        expected: [u8; 32],
        observed: [u8; 32],
    },
    /// The selected chunk frame fails its own C.11 integrity check, but the
    /// content does not yield a distinct trustworthy observed digest.
    ChunkFrameCorruption {
        ordinal: u64,
        record: PersistedRecordIdentity,
    },
    /// The selected C.5 outer frame failed before any inner digest could be
    /// observed. Other selected chunk records remain independently readable.
    ChunkOuterCorruption {
        ordinal: u64,
        record: PersistedRecordIdentity,
    },
}

/// Logical tree/chunk loads, caller bytes, and actual C5 traversal work.
/// Publication catalog selection is a separate protected B-tree point and is
/// excluded from these blob tree/chunk counters.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BlobReadObservation {
    touched_chunks: u64,
    tree_nodes_loaded: u64,
    returned_bytes: u64,
    physical_work_count: u64,
    reuse_source_selected_reads: u64,
}

impl BlobReadObservation {
    pub const fn touched_chunks(self) -> u64 {
        self.touched_chunks
    }

    pub const fn tree_nodes_loaded(self) -> u64 {
        self.tree_nodes_loaded
    }

    pub const fn returned_bytes(self) -> u64 {
        self.returned_bytes
    }

    /// C5 data-read and artifact-metadata operations admitted for tree and
    /// chunk traversal, including root validation at open. Residency hits
    /// and reuse of the retained chunk add no physical work.
    pub const fn physical_work_count(self) -> u64 {
        self.physical_work_count
    }

    /// Selected publication/tree/chunk checks behind a reuse claim.
    pub const fn reuse_source_selected_reads(self) -> u64 {
        self.reuse_source_selected_reads
    }

    fn observe_record_read(&mut self, observed: crate::physical_runtime::RecordReadObservation) {
        self.physical_work_count = self
            .physical_work_count
            .saturating_add(observed.physical_work_count());
    }
}

/// Reads one selected generation under the exact C5 root captured by its
/// publication scan. No caller-supplied digest or locator grants access.
pub struct BlobReadSession<'runtime> {
    pub(super) reader: PhysicalRecordReader,
    pub(super) publication: worth_store_physical_format::BlobGenerationPublicationV1,
    pub(super) cursor: u64,
    pub(super) end: u64,
    pub(super) nodes: Vec<traversal::SelectedBlobNode>,
    pub(super) chunk: Option<traversal::SelectedBlobChunk>,
    damaged_chunk_edge: Option<traversal::SelectedBlobChunkEdge>,
    pub(super) observation: BlobReadObservation,
    pub(super) _allocation: BlobPhysicalAllocation<'runtime>,
}

impl ServingPhysicalRuntime {
    /// Certification oracle from the selected C.5 root, independent of the
    /// derived BlobCatalog leaf used by the normal reader.
    #[cfg(feature = "certification-test-authority")]
    pub fn certification_selected_latest_blob_publication(
        &self,
    ) -> Result<
        Option<worth_store_physical_format::IndexedThroughBlobPublication>,
        PhysicalReadProtectionDenial,
    > {
        Ok(self.records()?.selected_latest_blob_publication())
    }

    /// Resolves a portable object/generation reference only through the
    /// currently selected C5 root. No serialized handle can mint read authority.
    pub(in crate::physical_runtime) fn resolve_blob_publication(
        &self,
        object: [u8; 16],
        generation: u64,
        scope: &AdmittedBlobScope,
        limits: BlobReadLimits,
    ) -> Result<PublishedBlobGeneration, BlobReadOpenFailure> {
        selection::resolve(self, object, generation, scope, limits)
    }

    /// Opens a bounded range read from one actual selected blob publication.
    /// The session retains the same protected root for every tree/chunk read.
    pub(in crate::physical_runtime) fn open_blob_read(
        &self,
        published: PublishedBlobGeneration,
        scope: &AdmittedBlobScope,
        offset: u64,
        length: u64,
        limits: BlobReadLimits,
    ) -> Result<BlobReadSession<'_>, BlobReadOpenFailure> {
        selection::open(self, published, scope, offset, length, limits)
    }
}

impl BlobReadSession<'_> {
    pub const fn observation(&self) -> BlobReadObservation {
        self.observation
    }

    pub const fn remaining_bytes(&self) -> u64 {
        self.end - self.cursor
    }

    pub const fn published_bytes(&self) -> u64 {
        self.publication.total_bytes()
    }

    /// Issues a selected-record diagnostic target only after this protected
    /// read found a direct chunk with typed inner corruption. The expected
    /// digest and occurrence come from its authenticated publication/tree
    /// edge, never from caller-provided bytes or the damaged chunk itself.
    pub fn damaged_chunk_scrub_target(
        &self,
    ) -> Result<
        Option<crate::physical_runtime::PhysicalIntegrityScrubTarget>,
        crate::physical_runtime::PhysicalIntegrityScrubRequestDenial,
    > {
        use crate::physical_runtime::integrity::SelectedRecordScrubBasis;
        use worth_store_physical_format::BLOB_CHUNK_FRAME_MAX_BYTES;
        use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

        let Some(edge) = self.damaged_chunk_edge else {
            return Ok(None);
        };
        let chunk_size = self.publication.chunk_size();
        let ordinal = edge.start / u64::from(chunk_size);
        let scope = PhysicalArtifactScope::blob_chunk_frame(
            self.reader.store_identity(),
            edge.entry.record(),
            PhysicalByteRange::new(0, BLOB_CHUNK_FRAME_MAX_BYTES as u64)
                .expect("nonzero chunk frame ceiling"),
        );
        crate::physical_runtime::PhysicalIntegrityScrubTarget::selected_record(
            scope,
            self.reader.protected_root(),
            SelectedRecordScrubBasis::BlobChunk {
                session: self.publication.session(),
                ordinal,
                chunk_size,
                digest: edge.entry.digest(),
                covered_bytes: edge.entry.covered_bytes(),
            },
        )
        .map(Some)
    }

    /// Copies only the touched authenticated chunks. A zero result means the
    /// requested range is complete (or the caller supplied an empty target).
    pub fn read_next(&mut self, target: &mut [u8]) -> Result<usize, BlobReadFailure> {
        traversal::read_next(self, target)
    }
}

pub(in crate::physical_runtime::blob) fn hold_chunk_for_relocation<'runtime>(
    runtime: &'runtime ServingPhysicalRuntime,
    published: PublishedBlobGeneration,
    scope: &AdmittedBlobScope,
    offset: u64,
    limits: BlobReadLimits,
) -> Result<super::placement::BlobMovementReadHold<'runtime>, BlobReadOpenFailure> {
    let mut read = selection::open(runtime, published, scope, offset, 1, limits)?;
    let (record, bytes) = traversal::authenticated_relocation_edge(&mut read, offset)
        .map_err(BlobReadOpenFailure::Read)?;
    Ok(super::placement::BlobMovementReadHold::from_selected(
        read,
        record,
        bytes,
        runtime.runtime_identity(),
    ))
}
